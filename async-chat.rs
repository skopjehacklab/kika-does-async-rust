#![allow(unused_variables)]

use tokio::io::AsyncWriteExt;
use tokio::net;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let (tx, _rx0) = broadcast::channel::<String>(100);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6667").await.unwrap();
    println!("Listening on tcp://0.0.0.0:6667");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(process_client(socket, addr, tx.clone()));
    }
}

async fn process_client(mut socket: net::TcpStream, addr: std::net::SocketAddr, sender: broadcast::Sender<String>) {
    println!("Connected client addr: {}", addr);

    let mut receiver = sender.subscribe();

    let (reader, mut writer) = socket.split();
    let codec = tokio_util::codec::LinesCodec::new_with_max_length(1024);
    let mut reader = tokio_util::codec::FramedRead::new(reader, codec);

    loop {
        // let mut line = String::with_capacity(1024);

        tokio::select! {

            line = reader.next() => match line {
                None => break,
                Some(Err(_)) => break,
                Some(Ok(line)) => {
                    sender.send(format!("{}: {}", addr, line.clone())).unwrap();
                    continue;
                },
            },

            Ok(msg) = receiver.recv() => {
                writer.write_all(msg.as_bytes()).await.unwrap();
            }
        }
    }
    println!("{} disconnected", addr);
}
