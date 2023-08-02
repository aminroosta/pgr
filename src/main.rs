mod test_helpers;
use anyhow::Result;
use serde::Deserialize;
use tokio_postgres::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

async fn start(client: &mut Client, sql: &str) -> Result<()> {
    let pgr_sql = include_str!("pgr.sql");
    let pgr_sql = pgr_sql.replace("PLACEHOLDER", sql);
    client.batch_execute(&pgr_sql).await?;
    Ok(())
}

#[tokio::test]
async fn test_add() -> Result<()> {
    let sql = include_str!("../test/sql/user.sql");
    let mut client = test_helpers::connect().await;
    start(&mut client, sql).await?;

    let procs = get_pg_procs(&mut client).await?;
    dbg!(&procs);

    Ok(())
}

#[derive(Deserialize, Debug)]
struct PgArg {
    name: String,
    mode: String,
    ty: String,
}

#[derive(Deserialize, Debug)]
struct PgProc {
    name: String,
    retset: bool,
    rettype: String,
    args: Vec<PgArg>,
}

async fn get_pg_procs(client: &mut Client) -> Result<Vec<PgProc>> {
    let rows = client
        .query("select * from pgr._pgr_procs('pgr')", &[])
        .await?;

    let mut procs = Vec::new();
    for row in rows {
        let mut proc = PgProc {
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
