use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use datafusion::execution::context::SessionContext;
use datafusion_expr::ScalarUDF;

use crate::commands::*;

async fn set_up() -> std::io::Result<SessionContext> {
    let ctx = SessionContext::new();
    let host = ScalarUDF::from(Host::new());
    ctx.register_udf(host.clone());
    ctx.register_udtf("ps", ps_table_func());
    ctx.register_udtf("uptime", uptime_table_func());
    ctx.register_udtf("who", who_table_func());
    ctx.register_udtf("ls", ls_table_func());
    ctx.register_udtf("stat", stat_table_func());
    ctx.register_udtf("df", df_table_func());
    ctx.register_udtf("du", du_table_func());
    ctx.register_udtf("blkid", blkid_table_func());
    ctx.register_udtf("env", env_table_func());
    ctx.register_udtf("date", date_table_func());
    ctx.register_udtf("dir", dir_table_func());
    ctx.register_udtf("dpkg_list", dpkg_list_table_func());
    ctx.register_udtf("file", file_table_func());
    ctx.register_udtf("find", find_table_func());
    ctx.register_udtf("free", free_table_func());
    Ok(ctx)
}

async fn eval_sql(ctx: &SessionContext, sql: String) -> std::io::Result<()> {
    let result = ctx.sql(&sql).await;
    match result {
        Ok(df) => {
            df.show().await.unwrap();
        }
        Err(e) => {
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
