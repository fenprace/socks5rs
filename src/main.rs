use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

async fn handle(
    request: Request<Body>,
    socket: &hyper::server::conn::AddrStream,
) -> Result<Response<Body>, hyper::Error> {
    println!("=====");
    println!("Income Request:");
    println!("{:?}", request);
    println!();

    let (parts, body) = request.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    let method = parts.method;
    let uri = parts.uri;

    let response = match method {
        Method::CONNECT => {
            let host = uri.authority().unwrap();
            // let host = uri.authority.unwrap().as_str();
            let socket = TcpStream::connect(host.as_str()).await.unwrap();
            Response::builder().status(200).body(Body::empty()).unwrap()
        }
        _ => {
            let mut real_request = Request::builder();
            for (name, value) in parts.headers.into_iter() {
                match name {
                    Some(name) => real_request = real_request.header(name, value),
                    None => {}
                };
            }

            let real_request = real_request
                .method(method)
                .uri(uri)
                .body(Body::from(body_bytes))
                .unwrap();

            println!("Outgo Request to Real Server:");
            println!("{:?}", real_request);
            println!();

            let client = hyper::Client::new();
            let real_response = client.request(real_request).await?;

            println!("Income Response from Real Server:");
            println!("{:?}", real_response);
            println!();

            let (real_parts, real_body) = real_response.into_parts();
            Response::from_parts(real_parts, real_body)
        }
    };

    // println!("Outgoing Request to Real Server:")
    println!("Outgo Response to Client:");
    println!("{:?}", response);
    println!();

    Ok(response)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 54000));

    let make_service = make_service_fn(|clientSocket| async move {
        let service = service_fn(|clientRequest: Request<Body>| async move {
            handle(clientRequest, clientSocket)
        });
        service
    });

    let server = Server::bind(&addr).serve(make_service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
