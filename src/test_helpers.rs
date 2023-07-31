#![allow(dead_code)]
#![allow(unused_imports)]
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{Client, Connection, NoTls, Socket};

pub fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + std::panic::UnwindSafe,
{
    let result = std::panic::catch_unwind(|| {
        test();
    });

    println!("Tearing down...");

    assert!(result.is_ok());
}

pub async fn connect() -> Client {
    let (client, connection) =
        tokio_postgres::connect("host=192.168.2.10 user=postgres password=postgres", NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    return client;
}
