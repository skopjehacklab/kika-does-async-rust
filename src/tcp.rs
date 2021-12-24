use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

pub async fn tcp_server(addr: &str, sender: broadcast::Sender<String>) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on tcp://{}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(process_client(stream, addr, sender.clone()));
    }
}

async fn process_client(mut socket: TcpStream, addr: SocketAddr, sender: broadcast::Sender<String>) {
    println!("Connected client addr: tcp://{}", addr);

    let mut receiver = sender.subscribe();

    let (reader, mut writer) = socket.split();
    let codec = LinesCodec::new_with_max_length(1024);
    let mut reader = FramedRead::new(reader, codec);

    loop {
        tokio::select! {

            line = reader.next() => match line {
                None => break,
                Some(Err(_)) => break,
                Some(Ok(line)) => {
                    sender.send(format!("{}: {}\n", addr, line)).unwrap();
                    continue;
                },
            },

            Ok(msg) = receiver.recv() => {
                writer.write_all(msg.as_bytes()).await.unwrap();
            }
        }
    }
    println!("tcp://{} disconnected", addr);
}
