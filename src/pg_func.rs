use anyhow::{anyhow, Result};
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
    pub method: String,
    pub path: Vec<String>,
    pub query: Vec<String>,
    pub retset: bool,
    pub rettype: String,
    pub args: Vec<PgArg>,
}

impl PgFunc {
    pub async fn from_db(client: &mut Client) -> Result<Vec<PgFunc>> {
        let rows = client
            .query("select * from pgr._pgr_functions('pgr')", &[])
            .await?;

        let mut funcs = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let (method, path, query) = match Self::parse_name(&name) {
                Ok((method, path, query)) => (method, path, query),
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };
            let mut func = PgFunc {
                name: row.get("name"),
                method: method,
                path: path,
                query: query,
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
    fn parse_name(name: &str) -> Result<(String, Vec<String>, Vec<String>)> {
        if let Some((method, url)) = name.split_once(' ') {
            let method = method.trim().to_string();
            match url.trim().split('?').collect::<Vec<_>>().as_slice() {
                [path, query] => {
                    let query = query.split('&').map(|q| q.to_string()).collect();
                    let path = Self::parse_path(path);
                    // dbg!((&method, &path, &query));
                    return Ok((method, path, query));
                }
                [path] => {
                    let path = Self::parse_path(path);
                    // dbg!((&method, &path));
                    return Ok((method, path, vec![]));
                }
                _ => return Err(anyhow!("Invalid function name: {}", name)),
            }
        }
        Err(anyhow!("Invalid function name: {}", name))
    }
    fn parse_path(path: &str) -> Vec<String> {
        path.trim_matches(' ')
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect()
    }
}

#[tokio::test]
async fn test_from_db() -> Result<()> {
    let sql = include_str!("../test/sql/user.sql");
    let mut client = crate::db_client::connect().await?;
    crate::db_client::reload(&mut client, sql).await?;

    let functions = PgFunc::from_db(&mut client).await?;
    assert_eq!(functions.len(), 8);

    Ok(())
}

#[tokio::test]
async fn test_parse_name() -> Result<()> {
    let result = PgFunc::parse_name("GET /user/:id?name&age")?;
    assert_eq!(
        result,
        (
            "GET".to_owned(),
            vec!["user".to_owned(), ":id".to_owned()],
            vec!["name".to_owned(), "age".to_owned()]
        )
    );
    Ok(())
}
