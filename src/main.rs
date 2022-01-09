mod handlers;
mod lib;

use crate::handlers::handle;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 54000));
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (socket, from) = listener.accept().await.unwrap();

        tokio::spawn(async move { handle(socket, from).await });
    }
}
