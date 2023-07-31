mod test_helpers;
use tokio_postgres::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(())
}

#[tokio::test]
async fn test_add() {
    let client = test_helpers::connect().await;
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await
        .unwrap();

    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
}
