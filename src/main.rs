use actix_web::client::{Client, ClientRequest};
use actix_web::{dev, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use std::net::ToSocketAddrs;
use url::Url;

// TODO: This forwarded implementation is incomplete as it only handles the inofficial
// X-Forwarded-For header but not the official Forwarded one.
fn create_forwarded_req(
    client: &web::Data<Client>,
    head: &dev::RequestHead,
    new_url: &str,
) -> ClientRequest {
    let forwarded_req = client.request_from(new_url, head).no_decompress();
    if let Some(addr) = head.peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    }
}

pub async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let head = req.head();
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
    println!("run proxy");

    let proxy_addr = "127.0.0.1";
    let proxy_port = 3000;
    let forward_url = Url::parse(&format!(
        "http://{}",
        ("127.0.0.1", 8080)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap()
    ))
    .unwrap();

    HttpServer::new(move || {
        App::new()
            .data(Client::new())
            .data(forward_url.clone())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(forward))
    })
    .bind((proxy_addr, proxy_port))?
    .system_exit()
    .start()
    .await
}
