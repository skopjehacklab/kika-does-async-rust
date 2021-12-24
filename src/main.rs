use tokio::sync::broadcast;
mod tcp;
mod ws;

#[tokio::main]
async fn main() {
    let (sender, _receiver) = broadcast::channel::<String>(100);

    let sender_tcp = sender.clone();
    let tcp_server = tokio::spawn(async move { crate::tcp::tcp_server("0.0.0.0:6667", sender_tcp).await });

    let sender_ws = sender.clone();
    let ws_server = tokio::spawn(async move { crate::ws::ws_server("0.0.0.0:6680", sender_ws).await });

    tokio::select! {
        _ = tcp_server => {}
        _ = ws_server => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}
