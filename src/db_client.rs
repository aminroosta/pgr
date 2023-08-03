use anyhow::Result;
use tokio_postgres::{Client, NoTls};

pub async fn connect() -> Result<Client> {
    let (client, connection) =
        tokio_postgres::connect("host=127.0.0.1 user=postgres password=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    return Ok(client);
}

pub async fn reload(client: &mut Client, sql: &str) -> Result<()> {
    let pgr_sql = include_str!("pgr.sql");
    let pgr_sql = pgr_sql.replace("PLACEHOLDER", sql);
    client.batch_execute(&pgr_sql).await?;
    Ok(())
}
