#![allow(dead_code)]
mod test_helpers;
use anyhow::Result;
use serde::Deserialize;
use tokio_postgres::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

async fn reload(client: &mut Client, sql: &str) -> Result<()> {
    let pgr_sql = include_str!("pgr.sql");
    let pgr_sql = pgr_sql.replace("PLACEHOLDER", sql);
    client.batch_execute(&pgr_sql).await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct PgArg {
    name: String,
    mode: String,
    ty: String,
}

#[derive(Deserialize, Debug)]
struct PgFunc {
    name: String,
    retset: bool,
    rettype: String,
    args: Vec<PgArg>,
}

async fn get_pg_functions(client: &mut Client) -> Result<Vec<PgFunc>> {
    let rows = client
        .query("select * from pgr._pgr_functions('pgr')", &[])
        .await?;

    let mut procs = Vec::new();
    for row in rows {
        let mut proc = PgFunc {
            name: row.get("name"),
            retset: row.get("retset"),
            rettype: row.get("rettype"),
            args: vec![],
        };

        let argtypes: Vec<String> = row.get("argtypes");
        let argmodes: Vec<String> = row.get("argmodes");
        let argnames: Vec<String> = row.get("argnames");

        (0..argtypes.len()).for_each(|i| {
            let arg = PgArg {
                name: argnames[i as usize].clone(),
                mode: match argmodes.len() {
                    0 => "in".to_string(),
                    _ => argmodes[i as usize].clone(),
                }
                .to_string(),
                ty: argtypes[i as usize].clone(),
            };
            proc.args.push(arg);
        });

        procs.push(proc);
    }

    Ok(procs)
}

#[tokio::test]
async fn test_get_pg_functions() -> Result<()> {
    let sql = include_str!("../test/sql/user.sql");
    let mut client = test_helpers::connect().await;
    reload(&mut client, sql).await?;

    let functions = get_pg_functions(&mut client).await?;
    dbg!(&functions);

    Ok(())
}
