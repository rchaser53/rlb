use actix_web::client::Client;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};

use std::net::TcpStream;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

mod req;
use crate::req::{create_forward_url, create_forwarded_req};

pub async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let original_url = req.uri();
    let head = req.head();
    let host = "127.0.0.1";

    let new_url = create_forward_url(&original_url, host, 8080);
    let forwarded_req = create_forwarded_req(&client, head, new_url.as_str());
    let mut res = match forwarded_req.send_body(body.clone()).await {
        Ok(res) => res,
        Err(err) => {
            let new_url = "http://127.0.0.1:8081";
            let forwarded_req = create_forwarded_req(&client, head, new_url);

            println!("{}", err);
            forwarded_req.send_body(body).await?
        }
    };

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

#[actix_rt::main]
pub async fn main() -> std::io::Result<()> {
    let _ = thread::spawn(|| loop {
        sleep(Duration::new(2, 0));
        if TcpStream::connect("127.0.0.1:8080").is_ok() {
            println!("running!");
        } else {
            println!("down!");
        }
    });

    println!("run proxy");
    let proxy_addr = "127.0.0.1";
    let proxy_port = 3000;

    HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(forward))
    })
    .bind((proxy_addr, proxy_port))?
    .system_exit()
    .start()
    .await
}
