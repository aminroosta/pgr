use anyhow::Result;
use serde::Deserialize;
use tokio_postgres::Client;

#[derive(Deserialize, Debug)]
pub struct PgArg {
    pub name: String,
    pub mode: String,
    pub ty: String,
}

#[derive(Deserialize, Debug)]
pub struct PgFunc {
    pub name: String,
    pub retset: bool,
    pub rettype: String,
    pub args: Vec<PgArg>,
}

impl PgFunc {
    async fn from_db(client: &mut Client) -> Result<Vec<PgFunc>> {
        let rows = client
            .query("select * from pgr._pgr_functions('pgr')", &[])
            .await?;

        let mut funcs = Vec::new();
        for row in rows {
            let mut func = PgFunc {
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
                func.args.push(arg);
            });

            funcs.push(func);
        }
        Ok(funcs)
    }
}

#[tokio::test]
async fn test_from_db() -> Result<()> {
    let sql = include_str!("../test/sql/user.sql");
    let mut client = crate::db_client::connect().await?;
    crate::db_client::reload(&mut client, sql).await?;

    let functions = PgFunc::from_db(&mut client).await?;
    dbg!(&functions);

    Ok(())
}
