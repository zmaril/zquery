// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use async_trait::async_trait;
use datafusion::arrow::array::{Float32Array, Int32Array, StringArray};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::Session;
use datafusion::datasource::function::TableFunctionImpl;
use datafusion::datasource::TableProvider;
use datafusion::error::Result;
use datafusion::physical_plan::memory::MemoryExec;
use datafusion::physical_plan::ExecutionPlan;
use datafusion_expr::{Expr, TableType};
use serde_json::Value;
use std::process::Command;
use std::sync::Arc;
use std::process::Stdio;
use std::io::Write;

/// Ps function that returns a list of processes, by default returns local processes, pass in a host to get processes from a remote machine.
///
/// Usage: `ps([host])`
///
struct ProcessTable {
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub command: String,
    pub cpu_percent: Option<f32>,
    pub mem_percent: Option<f32>,
    pub pid: i32,
    pub rss: i32,
    pub started: Option<String>,
    pub stat: Option<String>,
    pub time: String,
    pub tty: Option<String>,
    pub user: String,
    pub vsz: i32,
}

#[async_trait]
impl TableProvider for ProcessTable {
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
        let batches = self.batches.clone();
        Ok(Arc::new(MemoryExec::try_new(
            &[batches],
            TableProvider::schema(self),
            projection.cloned(),
        ).unwrap()))
    }
}

pub struct ProcessTableFunc {}

impl TableFunctionImpl for ProcessTableFunc {
    fn call(&self, _exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let batches = get_processes(None)?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("command", DataType::Utf8, false),
            Field::new("cpu_percent", DataType::Float32, true),
            Field::new("mem_percent", DataType::Float32, true),
            Field::new("pid", DataType::Int32, false),
            Field::new("rss", DataType::Int32, false),
            Field::new("started", DataType::Utf8, true),
            Field::new("stat", DataType::Utf8, true),
            Field::new("time", DataType::Utf8, false),
            Field::new("tty", DataType::Utf8, true),
        ]));
        let table = ProcessTable {
            schema,
            batches,
        };
        Ok(Arc::new(table))
    }
}

pub fn parse(ps_output: String) -> Vec<Process> {
    let mut cmd = Command::new("jc");
    cmd.arg("--ps");

    //pipe in the ps output
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("Failed to execute command");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin
        .write_all(ps_output.as_bytes())
        .expect("Failed to write to stdin");
    drop(stdin); // Drop the stdin handle to allow the child process to exit

    let output = child.wait_with_output().expect("Failed to read output");
    let jc_output = String::from_utf8(output.stdout).expect("Failed to read output");

    let json: Value = serde_json::from_str(&jc_output).expect("Failed to parse JSON");

    let processes = json
        .as_array()
        .unwrap()
        .iter()
        .map(|p| Process {
            command: p["command"].as_str().unwrap().to_string(),
            cpu_percent: p["cpu_percent"].as_f64().map(|x| x as f32),
            mem_percent: p["mem_percent"].as_f64().map(|x| x as f32),
            pid: p["pid"].as_i64().unwrap() as i32,
            rss: p["rss"].as_i64().unwrap() as i32,
            started: p["started"].as_str().map(|x| x.to_string()),
            stat: p["stat"].as_str().map(|x| x.to_string()),
            time: p["time"].as_str().unwrap().to_string(),
            tty: p["tty"].as_str().map(|tty| tty.to_string()),
            user: p["user"].as_str().unwrap().to_string(),
            vsz: p["vsz"].as_i64().unwrap() as i32,
        })
        .collect();

    processes
}

fn get_processes(host: Option<String>) -> Result<Vec<RecordBatch>> {
    // run ps aux command locally for now
    let ps_output = Command::new("ps")
        .arg("aux")
        .output()
        .expect("Failed to execute ps command");
    let ps_output = String::from_utf8(ps_output.stdout).expect("Failed to read ps output");
    let processes = parse(ps_output);

    let fields = vec![
        Field::new("command", DataType::Utf8, false),
        Field::new("cpu_percent", DataType::Float32, true),
        Field::new("mem_percent", DataType::Float32, true),
        Field::new("pid", DataType::Int32, false),
        Field::new("rss", DataType::Int32, false),
        Field::new("started", DataType::Utf8, true),
        Field::new("stat", DataType::Utf8, true),
        Field::new("time", DataType::Utf8, false),
        Field::new("tty", DataType::Utf8, true),
        Field::new("user", DataType::Utf8, false),
        Field::new("vsz", DataType::Int32, false),
    ];
    let commands: StringArray = StringArray::from(
        processes.iter().map(|p| p.command.clone()).collect::<Vec<String>>()
    );
    let cpu_percent: Float32Array = Float32Array::from(
        processes.iter().map(|p| p.cpu_percent.unwrap_or(0.0)).collect::<Vec<f32>>()
    );
    let mem_percent: Float32Array = Float32Array::from(
        processes.iter().map(|p| p.mem_percent.unwrap_or(0.0)).collect::<Vec<f32>>()
    );
    let pid: Int32Array = Int32Array::from(
        processes.iter().map(|p| p.pid).collect::<Vec<i32>>()
    );
    let rss: Int32Array = Int32Array::from(
        processes.iter().map(|p| p.rss).collect::<Vec<i32>>()
    );
    let started: StringArray = StringArray::from(
        processes.iter().map(|p| p.started.clone().unwrap_or("".to_string())).collect::<Vec<String>>()
    );
    let stat: StringArray = StringArray::from(
        processes.iter().map(|p| p.stat.clone().unwrap_or("".to_string())).collect::<Vec<String>>()
    );
    let time: StringArray = StringArray::from(
        processes.iter().map(|p| p.time.clone()).collect::<Vec<String>>()
    );
    let tty: StringArray = StringArray::from(
        processes.iter().map(|p| p.tty.clone().unwrap_or("".to_string())).collect::<Vec<String>>()
    );
    let user: StringArray = StringArray::from(
        processes.iter().map(|p| p.user.clone()).collect::<Vec<String>>()
    );
    let vsz: Int32Array = Int32Array::from(
        processes.iter().map(|p| p.vsz).collect::<Vec<i32>>()
    );

    let schema = Arc::new(Schema::new(fields));

    let batch = RecordBatch::try_new(schema.clone(), vec![
        Arc::new(commands),
        Arc::new(cpu_percent),
        Arc::new(mem_percent),
        Arc::new(pid),
        Arc::new(rss),
        Arc::new(started),
        Arc::new(stat),
        Arc::new(time),
        Arc::new(tty),
        Arc::new(user),
        Arc::new(vsz),
    ])?;
        
    Ok(vec![batch])
}