use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use datafusion::execution::context::SessionContext;
use crate::ps::*;
use std::sync::Arc;
use crate::commandtable::*;
use datafusion::arrow::datatypes::{Schema, Field, DataType};

async fn set_up() -> std::io::Result<SessionContext> {
    let ctx = SessionContext::new();
    ctx.register_udtf("ps", Arc::new(ProcessTableFunc {}));
    ctx.register_udtf("ps2", Arc::new(CommandTableFunc {
        command: vec!["ps".to_string(), "aux".to_string()],
        jc_parser: "ps".to_string(),
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
    }));
    Ok(ctx)
}

async fn eval_sql(ctx: &SessionContext, sql: String) -> std::io::Result<()> {
    let result = ctx.sql(&sql).await;
    match result {
        Ok(df) => {
            df.show().await.unwrap();
        }
        Err(e) => {
            dbg!(&e);
            println!("Error: {:?}", e);
        }
    }
    Ok(())
}
pub async fn cli_eval(sql: String) -> std::io::Result<()> {
    let ctx = set_up().await.unwrap();
    eval_sql(&ctx, sql).await.unwrap();
    Ok(())
}

pub async fn cli_repl() -> std::io::Result<()> {
    let ctx = set_up().await.unwrap();

    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                eval_sql(&ctx, line).await.unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    Ok(())
}
