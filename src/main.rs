use actix_web::client::Client;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};

use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

mod req;
use crate::req::{create_base_url, create_forward_url, create_forwarded_req};

lazy_static! {
    static ref CURRENT_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref SERVERS: Mutex<Vec<Server>> = {
        let base_info = vec![
            Server {
                url: create_base_url("127.0.0.1", 8000).to_string(),
                is_alive: true,
            },
            Server {
                url: create_base_url("127.0.0.1", 8001).to_string(),
                is_alive: true,
            },
            Server {
                url: create_base_url("127.0.0.1", 8002).to_string(),
                is_alive: true,
            },
        ];
        Mutex::new(base_info)
    };
}

pub struct Server {
    pub url: String,
    pub is_alive: bool,
}

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
    let count = CURRENT_INDEX.load(Ordering::SeqCst);
    println!("{:?}", count);

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

pub fn passive_check() {
    // create local state because I want not to use Mutex every time
    let mut urls: Vec<_> = {
        let servers = SERVERS.lock().unwrap();
        servers
            .iter()
            .map(|server| server.url.to_string())
            .collect()
    };
    let _ = thread::spawn(move || loop {
        sleep(Duration::new(10, 0));

        let mut remove_targets = vec![];
        for (index, url) in urls.iter().enumerate() {
            if TcpStream::connect(url).is_ok() {
                println!("{} is running!", url);
            } else {
                println!("{} is down!", url);
                remove_targets.push(index);
                let mut servers = SERVERS.lock().unwrap();
                servers[index].is_alive = false;
            }
        }

        // use reverse to remove item from largest.
        remove_targets.reverse();
        for index in remove_targets {
            urls.remove(index);
        }
    });
}

#[actix_rt::main]
pub async fn main() -> std::io::Result<()> {
    passive_check();

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
