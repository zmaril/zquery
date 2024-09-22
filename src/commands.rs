use crate::commandtable::*;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;
use datafusion::common::Result;

use std::any::Any;
use datafusion_expr::{ColumnarValue, Signature, Volatility};
use datafusion_expr::ScalarUDFImpl;

#[derive(Debug)]
pub struct Host {
    signature: Signature
}

impl Host {
    pub fn new() -> Self {
        Self {
            signature: Signature::uniform(1, vec![DataType::Utf8], Volatility::Immutable)
        }
    }
}

/// Implement the ScalarUDFImpl trait for AddOne
impl ScalarUDFImpl for Host {
    fn as_any(&self) -> &dyn Any { self }
    fn name(&self) -> &str { "host" }
    fn signature(&self) -> &Signature { &self.signature }
    fn return_type(&self, _args: &[DataType]) -> Result<DataType> {
      Ok(DataType::Utf8)
    }
    // The actual implementation would add one to the argument
    fn invoke(&self, args: &[ColumnarValue]) -> Result<ColumnarValue> {
        let args = ColumnarValue::values_to_arrays(args)?;

        let new_array = args[0].clone();
        Ok(ColumnarValue::Array(Arc::new(new_array)))
    }
}

pub fn ps_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["ps", "aux"],
        jc_parser: "ps",
        schema: Arc::new(Schema::new(vec![
            Field::new("user", DataType::Utf8, true),
            Field::new("pid", DataType::Int64, true),
            Field::new("vsz", DataType::Int64, true),
            Field::new("rss", DataType::Int64, true),
            Field::new("tt", DataType::Utf8, true),
            Field::new("stat", DataType::Utf8, true),
            Field::new("started", DataType::Utf8, true),
            Field::new("time", DataType::Utf8, true),
            Field::new("command", DataType::Utf8, true),
            Field::new("cpu_percent", DataType::Float64, true),
            Field::new("mem_percent", DataType::Float64, true),
        ])),
        is_result_array: true,
    })
}

pub fn uptime_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["uptime"],
        jc_parser: "uptime",
        schema: Arc::new(Schema::new(vec![
            Field::new("uptime", DataType::Utf8, true),
            Field::new("users", DataType::Int64, true),
            Field::new("load_1m", DataType::Float64, true),
            Field::new("load_5m", DataType::Float64, true),
            Field::new("load_15m", DataType::Float64, true),
            Field::new("time_hour", DataType::Int64, true),
            Field::new("time_minute", DataType::Int64, true),
            Field::new("time_second", DataType::Int64, true),
            Field::new("uptime_days", DataType::Int64, true),
            Field::new("uptime_hours", DataType::Int64, true),
            Field::new("uptime_minutes", DataType::Int64, true),
            Field::new("uptime_total_seconds", DataType::Int64, true),
        ])),
        is_result_array: false,
    })
}

pub fn who_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["who", "-a"],
        jc_parser: "who",
        schema: Arc::new(Schema::new(vec![
            Field::new("user", DataType::Utf8, true),
            Field::new("event", DataType::Utf8, true),
            Field::new("tty", DataType::Utf8, true),
            Field::new("time", DataType::Utf8, true), // Convert this to a date somehow
            Field::new("epoch", DataType::Int64, true),
        ])),
        is_result_array: true,
    })
}

pub fn ls_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["ls", "-lah"],
        jc_parser: "ls",
        schema: Arc::new(Schema::new(vec![
            Field::new("filename", DataType::Utf8, true),
            Field::new("flags", DataType::Utf8, true),
            Field::new("links", DataType::Int64, true),
            Field::new("owner", DataType::Utf8, true),
            Field::new("group", DataType::Utf8, true),
            Field::new("size", DataType::Int64, true),
            Field::new("date", DataType::Utf8, true),
        ])),
        is_result_array: true,
    })
}

pub fn stat_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["stat"],
        jc_parser: "stat",
        schema: Arc::new(Schema::new(vec![
            Field::new("file", DataType::Utf8, true),
            Field::new("unix_device", DataType::Int64, true),
            Field::new("inode", DataType::Int64, true),
            Field::new("flags", DataType::Utf8, true),
            Field::new("links", DataType::Int64, true),
            Field::new("user", DataType::Utf8, true),
            Field::new("group", DataType::Utf8, true),
            Field::new("rdev", DataType::Int64, true),
            Field::new("size", DataType::Int64, true),
            Field::new("access_time", DataType::Utf8, true),
            Field::new("modify_time", DataType::Utf8, true),
            Field::new("change_time", DataType::Utf8, true),
            Field::new("birth_time", DataType::Utf8, true),
            Field::new("block_size", DataType::Int64, true),
            Field::new("blocks", DataType::Int64, true),
            Field::new("unix_flags", DataType::Utf8, true),
            Field::new("access_time_epoch", DataType::Int64, true),
            Field::new("access_time_epoch_utc", DataType::Int64, true),
            Field::new("modify_time_epoch", DataType::Int64, true),
            Field::new("modify_time_epoch_utc", DataType::Int64, true),
            Field::new("change_time_epoch", DataType::Int64, true),
            Field::new("change_time_epoch_utc", DataType::Int64, true),
            Field::new("birth_time_epoch", DataType::Int64, true),
            Field::new("birth_time_epoch_utc", DataType::Int64, true),
        ])),
        is_result_array: true,
    })
}

pub fn df_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["df", "-h"],
        jc_parser: "df",
        schema: Arc::new(Schema::new(vec![
            Field::new("filesystem", DataType::Utf8, true),
            Field::new("512_blocks", DataType::Int64, true),
            Field::new("used", DataType::Int64, true),
            Field::new("available", DataType::Int64, true),
            Field::new("iused", DataType::Int64, true),
            Field::new("ifree", DataType::Int64, true),
            Field::new("mounted_on", DataType::Utf8, true),
            Field::new("capacity_percent", DataType::Int64, true),
            Field::new("iused_percent", DataType::Int64, true),
        ])),
        is_result_array: true,
    })
}

pub fn du_table_func() -> Arc<CommandTableFunc> {
    Arc::new(CommandTableFunc {
        command: vec!["du", "-h"],
        jc_parser: "du",
        schema: Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, true),
            Field::new("size", DataType::Int64, true),
        ])),
        is_result_array: true,
    })
}
