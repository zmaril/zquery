use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::json::ReaderBuilder;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::Session;
use datafusion::datasource::function::TableFunctionImpl;
use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::physical_plan::memory::MemoryExec;
use datafusion::physical_plan::ExecutionPlan;
use datafusion_expr::{Expr, TableType};
use ssh2_config::{ParseRule, SshConfig};
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

/// Generic CommandTable that runs a command, pipes its output into `jc`, and provides the data as RecordBatches.

#[derive(Debug, Clone)]
pub struct CommandTable {
    command: Vec<String>,
    jc_parser: String,
    schema: SchemaRef,
    is_result_array: bool,
    hostname: String,
    args: Vec<String>,
}

impl CommandTable {
    fn run_command_remotely(
        host: &str,
        command: &[String],
        args: &[String],
        jc_parser: &str,
    ) -> std::io::Result<String> {

        let home = std::env::var("HOME").unwrap();

        let mut reader = BufReader::new(
            File::open(Path::new(&format!("{}/.ssh/config", home)))
                .expect("Could not open configuration file"),
        );

        let config = SshConfig::default()
            .parse(&mut reader, ParseRule::STRICT)
            .expect("Failed to parse configuration");

        // Query parameters for your host
        // If there's no rule for your host, default params are returned
        let params = config.query(host);

        let mut session = ssh2::Session::new().expect("Failed to create session");

        let hostname = params.host_name.unwrap();
        let port = params.port.unwrap_or(22);
        let user = params.user.unwrap_or("root".to_string());
        let tcp = TcpStream::connect(format!("{}:{}", hostname, port)).unwrap();

        session.set_tcp_stream(tcp);
        session.handshake().unwrap();
        // resolve the home directory
        session
            .userauth_pubkey_file(&user, None, Path::new(&format!("{}/.ssh/{}", home, "id_rsa")), None)
            .unwrap();

        let mut channel = session.channel_session().unwrap();
        //create the command string
        let mut command_string = command[0].clone();
        for arg in &command[1..] {
            command_string.push_str(&format!(" {}", arg));
        }
        for arg in args {
            command_string.push_str(&format!(" {}", arg));
        }
        channel.exec(&command_string).unwrap();

        // sleep for 3 seconds
        std::thread::sleep(std::time::Duration::from_secs(3));
        let mut s = String::new();

        loop {
            let mut buffer = [0; 1024]; // Buffer to hold incoming data
            let n = channel.read(&mut buffer).unwrap_or(0);

            if n == 0 {
                break;
            }

            s.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }

        // Ensure the channel closes cleanly
        channel.wait_close().unwrap();

        // Pipe the output into jc
        let mut jc_cmd = Command::new("jc");
        jc_cmd.arg(format!("--{}", jc_parser));
        jc_cmd.stdin(Stdio::piped());
        jc_cmd.stdout(Stdio::piped());
        let mut jc_child = jc_cmd.spawn()?;

        // Write the output of the initial command into jc's stdin
        let mut jc_stdin = jc_child.stdin.take().expect("Failed to open jc stdin");
        jc_stdin.write_all(s.as_bytes()).unwrap();
        // send a newline
        jc_stdin.write_all(b"\n").unwrap();
        drop(jc_stdin);

        // Read jc's output
        let output = jc_child.wait_with_output().unwrap();
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(stdout)
    }

    fn run_command_locally(command: &[String], args: &[String], jc_parser: &str) -> std::io::Result<String> {
        // Run the initial command
        let mut cmd = Command::new(&command[0]);
        if command.len() > 1 {
            cmd.args(&command[1..]);
        }
        if args.len() > 0 {
            cmd.args(args);
        }
        cmd.stdout(Stdio::piped());
        let child = cmd.spawn()?;

        // Pipe the output into jc
        let mut jc_cmd = Command::new("jc");
        jc_cmd.arg(format!("--{}", jc_parser));
        jc_cmd.stdin(Stdio::piped());
        jc_cmd.stdout(Stdio::piped());
        let mut jc_child = jc_cmd.spawn()?;

        // Write the output of the initial command into jc's stdin
        let mut jc_stdin = jc_child.stdin.take().expect("Failed to open jc stdin");
        let mut child_stdout = child.stdout.expect("Failed to open command stdout");

        std::thread::spawn(move || {
            let mut buffer = [0; 4096];
            loop {
                let n = match child_stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };
                if jc_stdin.write_all(&buffer[..n]).is_err() {
                    break;
                }
            }
        });

        // Read jc's output
        let output = jc_child.wait_with_output()?;
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(stdout)
    }
}

fn json_to_ndjson(json: &str) -> String {
    //run it through jq
    let mut cmd = Command::new("jq");
    cmd.arg("-c");
    cmd.arg(".[]");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    {
        let mut stdin = child.stdin.take().expect("Failed to open jq stdin");
        stdin.write_all(json.as_bytes()).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[async_trait]
impl TableProvider for CommandTable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // Run the command and parse the output
        let mut output = if self.hostname == "localhost" {
            Self::run_command_locally(&self.command, &self.args, &self.jc_parser).map_err(|e| {
                DataFusionError::Execution(format!("Failed to execute command: {}", e))
            })?
        } else {
            Self::run_command_remotely(&self.hostname, &self.command, &self.args, &self.jc_parser).map_err(
                |e| DataFusionError::Execution(format!("Failed to execute command: {}", e)),
            )?
        };

        if self.is_result_array {
            output = json_to_ndjson(&output);
        }

        let cursor = Cursor::new(output);
        let reader = ReaderBuilder::new(self.schema.clone())
            .build(cursor)
            .map_err(|e| {
                DataFusionError::Execution(format!("Failed to build JSON reader: {}", e))
            })?;
        let batches: Vec<RecordBatch> = reader
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DataFusionError::Execution(format!("Failed to read batches: {}", e)))?;

        Ok(Arc::new(MemoryExec::try_new(
            &[batches],
            self.schema.clone(),
            projection.cloned(),
        )?))
    }
}

#[derive(Debug, Clone)]
pub struct CommandTableFunc {
    pub command: Vec<&'static str>,
    pub jc_parser: &'static str,
    pub schema: SchemaRef,
    pub is_result_array: bool,
}

fn get_values_from_literals(exprs: &[Expr]) -> Vec<String> {
        exprs.iter().map(|e|  {
            match e {
                Expr::Literal(lit) => lit.to_string(),
                _ => "".to_string(),
            }
        }).collect()
}

impl TableFunctionImpl for CommandTableFunc {

    fn call(&self, exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let mut hostname = "localhost".to_string();
        let mut args = Vec::new();
        // eventually these need to evaled
        if !exprs.is_empty() {
            match &exprs[0] {
                Expr::ScalarFunction(func) => {
                    //todo, make sure that the function is the host function
                    if let Expr::Literal(lit) = &func.args[0] {
                        hostname = lit.to_string();
                    }
                    args = get_values_from_literals(&exprs[1..]);
                }
                _ => {
                    args = get_values_from_literals(exprs);
                }
            }
        }

        let table = CommandTable {
            command: self.command.iter().map(|s| s.to_string()).collect(),
            jc_parser: self.jc_parser.to_string(),
            schema: self.schema.clone(),
            is_result_array: self.is_result_array,
            hostname,
            args,
        };
        Ok(Arc::new(table))
    }
}
