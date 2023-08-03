#![allow(dead_code)]
mod db_client;
mod pg_func;

use anyhow::Result;
use pg_func::{PgArg, PgFunc};
use tokio_postgres::Client;
use warp::{http::HeaderMap, Filter};

#[tokio::main]
async fn main() -> Result<()> {
    let pg_func = PgFunc {
        name: "get /user/:id?name&age".to_string(),
        retset: false,
        rettype: "pgr.user_t".to_string(),
        args: vec![
            PgArg {
                name: "id".to_string(),
                mode: "in".to_string(),
                ty: "int4".to_string(),
            },
            PgArg {
                name: "name".to_string(),
                mode: "in".to_string(),
                ty: "text".to_string(),
            },
            PgArg {
                name: "age".to_string(),
                mode: "in".to_string(),
                ty: "int4".to_string(),
            },
        ],
    };

    let mut client = db_client::connect().await?;
    handle_pg_func(&mut client, &pg_func).await?;

    // get /user?id&name
    let hello = warp::path!("hello" / String)
        .and(warp::header::headers_cloned())
        // GET /hello/warp => 200 OK with body "Hello, warp!"
        .map(|name: String, headers: HeaderMap| {
            let headers_vec = headers
                .iter()
                .map(|(k, v)| vec![k.as_str(), v.to_str().unwrap()])
                .flatten()
                .collect::<Vec<_>>();
            dbg!(&headers_vec);
            format!("Hello, {}\n{:?}!", name, headers)
        });

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

async fn handle_pg_func(_client: &mut Client, func: &PgFunc) -> Result<()> {
    let name = func.name.split_whitespace().collect::<Vec<_>>();
    if !name.len() == 2 {
        println!("Invalid function name: {}", func.name);
        return Ok(());
    }
    let verb = name[0];
    if !["get", "put", "post", "patch", "delete"].contains(&verb) {
        println!("invalid function name: {}", func.name);
        return Ok(());
    }
    let url = name[1].split("?").collect::<Vec<_>>();
    if !url.len() <= 2 {
        println!("Invalid function name: {}", func.name);
        return Ok(());
    }
    let path = url[0];
    let query = if url.len() == 2 { url[1] } else { "" };

    dbg!(&verb, &path, &query);
    dbg!(&func.args);

    Ok(())
}
