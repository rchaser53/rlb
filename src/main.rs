use actix_web::client::{Client, ClientResponse, SendRequestError};
use actix_web::{
    dev::{Decompress, Payload, PayloadStream, RequestHead},
    middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use url::Url;

use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

mod req;
use crate::req::{create_base_url, create_forwarded_req};

lazy_static! {
    static ref CURRENT_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref SERVERS: Mutex<Vec<Server>> = {
        let base_info = vec![
            Server {
                url: create_base_url("127.0.0.1", 8000),
                is_alive: true,
            },
            Server {
                url: create_base_url("127.0.0.1", 8001),
                is_alive: true,
            },
            Server {
                url: create_base_url("127.0.0.1", 8002),
                is_alive: true,
            },
        ];
        Mutex::new(base_info)
    };
}

pub struct Server {
    pub url: Url,
    pub is_alive: bool,
}

fn get_new_url() -> String {
    let mut current_index = CURRENT_INDEX.fetch_add(1, Ordering::SeqCst);
    if current_index > 100 {
        CURRENT_INDEX.store(0, Ordering::SeqCst);
    }

    let servers = SERVERS.lock().unwrap();
    let length = servers.len();

    while !servers[current_index % length].is_alive {
        current_index += 1;
        if servers.iter().all(|server| !server.is_alive) {
            panic!("all server are down!");
        }
    }
    servers[current_index % length].url.to_string()
}

pub async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let head = req.head();
    let mut res;
    loop {
        if let Ok(raw_res) = active_check(&client, head, &body, get_new_url().as_str()).await {
            res = raw_res;
            break;
        }
    }

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

pub async fn active_check(
    client: &web::Data<Client>,
    head: &RequestHead,
    body: &web::Bytes,
    new_url: &str,
) -> Result<ClientResponse<Decompress<Payload<PayloadStream>>>, SendRequestError> {
    let retry_count: usize = 3;
    let mut index = 0;
    loop {
        let forwarded_req = create_forwarded_req(&client, head, new_url);
        let res_result = forwarded_req.send_body(body.clone()).await;
        match res_result {
            Ok(raw_res) => return Ok(raw_res),
            Err(err) => {
                println!("{}", &err);
                if index >= retry_count {
                    return Err(err);
                }
            }
        }
        index += 1;
    }
}

pub fn passive_check() {
    // create local state because I want not to use Mutex every time
    let mut host_and_ports: Vec<_> = {
        let servers = SERVERS.lock().unwrap();
        servers
            .iter()
            .map(|server| {
                format!(
                    "{}:{}",
                    server.url.host_str().unwrap(),
                    server.url.port().unwrap()
                )
            })
            .collect()
    };
    let _ = thread::spawn(move || loop {
        sleep(Duration::new(2, 0));

        let mut remove_targets = vec![];
        for (index, host_and_port) in host_and_ports.iter().enumerate() {
            match TcpStream::connect(host_and_port) {
                Ok(_) => {
                    println!("{} is running!", host_and_port);
                }
                Err(err) => {
                    println!("{}", err);
                    println!("{} is down!", host_and_port);
                    remove_targets.push(index);
                    let mut servers = SERVERS.lock().unwrap();
                    servers[index].is_alive = false;
                }
            }
        }

        // use reverse to remove item from largest.
        remove_targets.reverse();
        for index in remove_targets {
            host_and_ports.remove(index);
        }

        if host_and_ports.len() == 0 {
            panic!("all server are down!");
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
