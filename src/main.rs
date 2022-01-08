mod lib;

use crate::lib::{get_port, S5Addr, S5Request};
use log::*;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 54000));
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (mut socket, from) = listener.accept().await.unwrap();

        info!("Income Connection from {:?}", from);

        handle_handeshake(&mut socket).await;
        let req = handle_connection(&mut socket).await.unwrap();
        let real_socket = handle_traffic(&mut socket, req).await.unwrap();

        let mut bl = [0u8; 1024];
        let mut br = [0u8; 1024];

        let (mut rs, mut ws) = socket.into_split();
        let (mut rr, mut wr) = real_socket.into_split();

        tokio::spawn(async move {
            loop {
                match rs.read(&mut bl).await {
                    Err(e) => {
                        error!("Failed to Read from Real Server: {}", e);
                        break;
                    }
                    Ok(0) => {
                        break;
                    }
                    _ => {}
                }

                if let Err(e) = wr.write_all(&bl).await {
                    error!("Failed to Write to Client: {}", e);
                    break;
                }
            }
        });

        loop {
            match rr.read(&mut br).await {
                Err(e) => {
                    error!("Failed to Read from Client: {}", e);
                    break;
                }
                Ok(0) => {
                    break;
                }
                _ => {}
            }

            if let Err(e) = ws.write_all(&br).await {
                error!("Failed to Write to Real Server: {}", e);
                break;
            }
        }
    }
}

async fn handle_handeshake(socket: &mut TcpStream) {
    let mut buf: [u8; 32] = [0; 32];
    read(socket, &mut buf).await;

    if buf[0] != 0x05 {
        panic!("Not Socks5 Connection");
    }

    write(socket, b"\x05\x00").await;
}

async fn handle_connection(socket: &mut TcpStream) -> Option<S5Request> {
    let mut buf: [u8; 32] = [0; 32];
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
            // println!("Client tries to access: {}", dest);

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

async fn read(socket: &mut TcpStream, buf: &mut [u8]) {
    socket.read(buf).await.unwrap();
    info!("Income Data: {:?}", buf);
}

async fn write(socket: &mut TcpStream, response: &[u8]) {
    info!("Outgo Data: {:?}", response);
    socket.write_all(response).await.unwrap();
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
