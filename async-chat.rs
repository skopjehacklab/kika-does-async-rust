#![allow(unused_variables)]

use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net;
use tokio::sync::broadcast;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6667").await.unwrap();
    println!("Listening on :6667");

    let (tx, _rx0) = broadcast::channel::<String>(100);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(process_client(socket, addr, tx.clone()));
        // do rx0
    }
}

async fn process_client(mut socket: net::TcpStream, addr: std::net::SocketAddr, sender: broadcast::Sender<String>) {
    println!("Connected client addr: {}", addr);

    let mut receiver = sender.subscribe();

    let (reader, mut writer) = socket.split();
    let mut reader = tokio::io::BufReader::new(reader);

    loop {
        let mut line = String::with_capacity(1024); // [u8]

        tokio::select! {
            //                                   Future<Result<usize>>>
            t = timeout(Duration::from_secs(60), reader.read_line(&mut line)) => {
                let x = match t {
                    Err(_) => break,
                    Ok(Ok(0)) => break,
                    Ok(Ok(1)) => break,
                    Ok(Ok(_)) => { sender.send(line.clone()).unwrap(); continue },
                    Ok(Err(e)) => break,
                };
            },

            Ok(msg) = receiver.recv() =>
                writer.write_all(msg.as_bytes()).await.unwrap(),
        }
    }
    println!("{} disconnected", addr);
}
