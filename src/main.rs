mod test_helpers;
use anyhow::Result;
use tokio_postgres::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

async fn start(client: &mut tokio_postgres::Client, sql: &str) -> Result<()> {
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
async fn test_add() {
    let sq = r#"
        create type user_t AS (
          user_id integer,
          name text
        );

        create function "get /user/:id"(id integer) returns user_t as $$
          select * from users where user_id = id;
        $$ language sql;

        create function "post /user"(body user_t) returns user_t as $$
          insert into users values (body.*) returning *;
        $$ language sql;

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
    start(&mut client, sq).await.unwrap();
    let result = client
        .query(
            r#"
            select proname
            from pg_catalog.pg_proc
            join pg_catalog.pg_namespace on pg_namespace.oid = pg_proc.pronamespace
            where nspname = 'pgr'
            "#,
            &[],
        )
        .await
        .unwrap();
    dbg!(&result[0].get::<_, String>(0));
}
