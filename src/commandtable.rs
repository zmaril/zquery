use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::Session;
use datafusion::datasource::TableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::physical_plan::memory::MemoryExec;
use datafusion::physical_plan::ExecutionPlan;
use datafusion_expr::{Expr, TableType};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use datafusion::arrow::json::ReaderBuilder;
use std::io::Cursor;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::function::TableFunctionImpl;

/// Generic CommandTable that runs a command, pipes its output into `jc`, and provides the data as RecordBatches.

#[derive(Debug, Clone)]
pub struct CommandTable {
    command: Vec<String>,
    jc_parser: String,
    schema: SchemaRef,
    is_result_array: bool,
}

impl CommandTable {
    fn run_command(command: &[String], jc_parser: &str) -> std::io::Result<String> {
        // Run the initial command
        let mut cmd = Command::new(&command[0]);
        if command.len() > 1 {
            cmd.args(&command[1..]);
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
    {let mut stdin = child.stdin.take().expect("Failed to open jq stdin");
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
        let mut output = Self::run_command(&self.command, &self.jc_parser)
            .map_err(|e| DataFusionError::Execution(format!("Failed to execute command: {}", e)))?;

        if self.is_result_array {
            output = json_to_ndjson(&output);
        }

        let cursor = Cursor::new(output);
        let reader = ReaderBuilder::new(self.schema.clone())
            .build(cursor)
            .map_err(|e| DataFusionError::Execution(format!("Failed to build JSON reader: {}", e)))?;
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

impl TableFunctionImpl for CommandTableFunc {
    fn call(&self, _exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let table = CommandTable {
            command: self.command.iter().map(|s| s.to_string()).collect(),
            jc_parser: self.jc_parser.to_string(),
            schema: self.schema.clone(),
            is_result_array: self.is_result_array,
        };
        Ok(Arc::new(table))
    }
}