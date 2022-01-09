use crate::lib::{S5Addr, S5Request};
use log::*;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

async fn read(socket: &mut TcpStream, buf: &mut [u8]) {
    socket.read(buf).await.unwrap();
    info!("Income Data: {:?}", buf);
}

async fn write(socket: &mut TcpStream, response: &[u8]) {
    info!("Outgo Data: {:?}", response);
    socket.write_all(response).await.unwrap();
}

async fn copy(from: &mut OwnedReadHalf, to: &mut OwnedWriteHalf) {
    let mut buf = [0u8; 1024];
    loop {
        let len = from.read(&mut buf).await;
        if let Err(e) = len {
            error!("Failed to Read: {}", e);
            break;
        } else if let Ok(0) = len {
            break;
        } else if let Ok(len) = len {
            if let Err(e) = to.write_all(&buf[..len]).await {
                error!("Failed to Write: {}", e);
                break;
            }
        }
    }
}

pub fn get_port(buf: &[u8], start: usize) -> u16 {
    (buf[start] as u16) << 8 | (buf[start + 1] as u16)
}

pub async fn handle(socket: TcpStream, from: SocketAddr) {
    info!("Income Connection from {}", from);
    let mut socket = socket;

    handle_handeshake(&mut socket).await.unwrap();
    let req = handle_connection(&mut socket).await.unwrap();
    let real_socket = handle_traffic(&mut socket, req).await.unwrap();

    let (mut rs, mut ws) = socket.into_split();
    let (mut rr, mut wr) = real_socket.into_split();

    tokio::spawn(async move {
        copy(&mut rs, &mut wr).await;
    });

    tokio::spawn(async move {
        copy(&mut rr, &mut ws).await;
    });
}

async fn handle_handeshake(socket: &mut TcpStream) -> Option<()> {
    let mut buf = [0u8; 32];
    read(socket, &mut buf).await;

    if buf[0] != 0x05 {
        error!("Not Socks5 Connection");
        return None;
    }

    write(socket, b"\x05\x00").await;
    Some(())
}

async fn handle_connection(socket: &mut TcpStream) -> Option<S5Request> {
    let mut buf = [0u8; 32];
    read(socket, &mut buf).await;

    let atyp = buf[3];

    let dst = match atyp {
        0x01 => {
            let dst_addr = S5Addr::IPv4(buf[4], buf[5], buf[6], buf[7]);
            let dst_port = get_port(&buf, 8);
            Some((dst_addr, dst_port))
        }
        0x03 => {
            let start = 5;
            let length = buf[4];
            let end: usize = (start + length) as usize;

            let dst_addr = S5Addr::Domain(String::from(std::str::from_utf8(&buf[5..end]).unwrap()));
            let dst_port = get_port(&buf, end);
            Some((dst_addr, dst_port))
        }
        _ => None,
    };

    match dst {
        None => None,
        Some((dst_addr, dst_port)) => {
            let req = S5Request::new(buf[0], buf[1], buf[3], dst_addr, dst_port);

            if req.cmd != 0x01 {
                warn!("Unsupported CMD: {:?}", req.cmd);
                write(socket, b"\x05\x07\x00\x01\x00\x00\x00\x00\x00\x00").await;
                return None;
            }

            if req.atype != 0x01 {
                warn!("Unsupported ATYP: {:?}", req.atype);
                write(socket, b"\x05\x08\x00\x01\x00\x00\x00\x00\x00\x00").await;
                return None;
            }

            Some(req)
        }
    }
}

async fn handle_traffic(socket: &mut TcpStream, req: S5Request) -> Option<TcpStream> {
    let addr = req.into_addr_string();
    info!("Client Tries to Connect to {}", addr);

    match TcpStream::connect(&addr).await {
        Err(e) => {
            warn!("Failed to Connect with Real Server: {}", e);
            write(socket, b"\x05\x04\x00\x01\x00\x00\x00\x00\x00\x00").await;
            None
        }
        Ok(real_socket) => {
            info!("Connected to Real Server: {}", &addr);
            write(socket, b"\x05\x00\x00\x01\x00\x00\x00\x00\x00\x00").await;
            Some(real_socket)
        }
    }
}
