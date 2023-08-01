mod test_helpers;
use anyhow::Result;
use serde::Deserialize;
use tokio_postgres::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

async fn start(client: &mut Client, sql: &str) -> Result<()> {
    let sql_wrapped = format!(
        r#"
        begin;
        drop schema if exists pgr cascade;
        create schema pgr;
        set search_path to pgr, public;
        {}
        commit;
        "#,
        sql
    );
    client.batch_execute(&sql_wrapped).await?;
    Ok(())
}

#[tokio::test]
async fn test_add() -> Result<()> {
    let sq = r#"
        create type user_t AS (
          user_id integer,
          name text
        );

        create function "get /user/:id"(id integer) returns user_t as $$
          select * from users where user_id = id;
        $$ language sql;

        create function "post /user"(
          body user_t,
          out response user_t
        ) as $$
          begin
            insert into users values (body.*) returning * into response;
          end;
        $$ language plpgsql;


        create function "put /user/:id"(id integer, body user_t) returns user_t as $$
          update users set name = body.name where user_id = id returning *;
        $$ language sql;

        create function "get /users"() returns setof user_t as $$
          select * from users;
        $$ language sql;

        create function "get /users/count"() returns integer as $$
          select count(*) from users;
        $$ language sql;
    "#;
    let mut client = test_helpers::connect().await;
    start(&mut client, sq).await?;

    let procs = get_pg_procs(&mut client).await?;
    dbg!(&procs);

    Ok(())
}

#[derive(Deserialize, Debug)]
struct PgArg {
    name: String,
    mode: String,
    ty: u32,
}

#[derive(Deserialize, Debug)]
struct PgProc {
    name: String,
    retset: bool,
    rettype: u32,
    args: Vec<PgArg>,
}

async fn get_pg_procs(client: &mut Client) -> Result<Vec<PgProc>> {
    let rows = client
        .query(
            "select
            proname name,
            proretset retset,
            prorettype rettype,
            coalesce(proallargtypes, proargtypes) as argtypes,
            coalesce(proargmodes, '{}') as argmodes,
            coalesce(proargnames, '{}') argnames
            from pg_proc
          join pg_namespace on pg_namespace.oid = pg_proc.pronamespace
          where
            nspname = 'pgr' and
            prokind = 'f' and
            provariadic = 0 -- variadic functions are not supported
          ;",
            &[],
        )
        .await?;

    let mut procs = Vec::new();
    for row in rows {
        let mut proc = PgProc {
            name: row.get("name"),
            retset: row.get("retset"),
            rettype: row.get("rettype"),
            args: vec![],
        };
        let argtypes: Vec<u32> = row.get("argtypes");
        let argmodes: Vec<i8> = row.get("argmodes");
        let argnames: Vec<String> = row.get("argnames");

        (0..argtypes.len()).for_each(|i| {
            let arg = PgArg {
                name: argnames[i as usize].clone(),
                mode: match argmodes.len() {
                    0 => "in",
                    _ => match argmodes[i as usize] {
                        105 => "in",
                        111 => "out",
                        98 => "inout",
                        118 => "variadic",
                        116 => "table",
                        _ => panic!("unknown argmode {i}"),
                    },
                }
                .to_string(),
                ty: argtypes[i as usize],
            };
            proc.args.push(arg);
        });

        procs.push(proc);
    }

    Ok(procs)
}
