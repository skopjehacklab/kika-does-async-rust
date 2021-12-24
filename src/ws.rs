use futures_util::{stream::StreamExt, SinkExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_tungstenite::tungstenite::Message;

pub async fn ws_server(addr: &str, sender: broadcast::Sender<String>) {
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind websocket");
    println!("Listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, sender.clone()));
    }
}

async fn accept_connection(stream: TcpStream, sender: broadcast::Sender<String>) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    println!("Connected client addr: ws://{}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let mut receiver = sender.subscribe();

    let (mut writer, mut reader) = ws_stream.split();
    loop {
        tokio::select! {

            msg = reader.next() => match msg {
                None => break,
                Some(Err(_)) => break,
                Some(Ok(msg)) => {
                    sender.send(format!("{}: {}\n", addr, msg)).unwrap();
                    continue;
                },
            },

            Ok(msg) = receiver.recv() => {
                writer.send(Message::text(msg)).await.unwrap();
            }
        }
    }
    println!("ws://{} disconnected", addr);
}
