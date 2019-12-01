// #[macro_use]
extern crate crossbeam_channel;

#[macro_use]
extern crate actix_rt;

#[macro_use]
extern crate actix_web;

// use actix_web::{get, middleware, web, App, Error, HttpRequest, Responder, HttpServer};
use actix_web::client::Client;
use actix_web::{get, middleware, web, web::Bytes, App, HttpRequest, HttpResponse, HttpServer, Responder};

// use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use std::env::set_var;

use futures::stream::Stream;
use futures::Future;
use std::net::ToSocketAddrs;
use url::Url;

// use std::io;
// use std::sync::{Arc, Mutex};

// u, _ := url.Parse("http://localhost:8080")
// rp := httputil.NewSingleHostReverseProxy(u)

// // initialize your server and add this as handler
// http.HandlerFunc(rp.ServeHTTP)
// use actix_web::{web, App, HttpServer, Responder};

#[get("/resource1/{name}/index.html")]
async fn index(req: HttpRequest, name: web::Path<String>) -> String {
    println!("REQ: {:?}", req);
    format!("Hello: {}!\r\n", name)
}

async fn index_async(req: HttpRequest) -> &'static str {
    println!("REQ: {:?}", req);
    "Hello world!\r\n"
}

#[get("/")]
async fn no_params() -> &'static str {
    "Hello world!\r\n"
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let _ = thread::scope(|scope| {
        scope.spawn(|_| server("127.0.0.1:8081"));
        scope.spawn(|_| server("127.0.0.1:8080"));
    })
    .unwrap();   
}

fn server(bind_url: &str) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(no_params)
            .service(
                web::resource("/resource2/index.html")
                    .wrap(middleware::DefaultHeaders::new().header("X-Version-R2", "0.3"))
                    .default_service(web::route().to(|| HttpResponse::MethodNotAllowed()))
                    .route(web::get().to(index_async)),
            )
            .service(web::resource("/test1.html").to(|| async { "Test\r\n" }))
    })
    .bind(bind_url)?
    .workers(1)
    .run()
}

async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    url: web::Data<Url>,
    client: web::Data<Client>,
// ) -> impl Future<Item = HttpResponse, Error = Error> {
// ) -> impl Future<Output = Responder> {
// ) -> Result<impl Future<Output = HttpResponse>, actix_web::client::SendRequestError> {
) -> HttpResponse {
    let url = Url::parse("http://127.0.0.1:8080").unwrap();
    let url = web::Data::new(url);

    let mut new_url = url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    let a = forwarded_req
        .send_body(body)
        .await
        // .map_err(Error::from)
        .map(|mut res| async {
            let mut client_resp = HttpResponse::build(res.status());
            // Remove `Connection` as per
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
            for (header_name, header_value) in
                res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }

            client_resp
              .streaming(res)

            // res.body()
            //     .into_stream()
            //     .concat2()
            //     .map(move |b| client_resp.body(b))
            //     .map_err(|e| e.into())
        })
        .unwrap()
        .await;
        // .flatten()
    a
}

fn ccc() -> std::io::Result<()> {
    let listen_addr = "127.0.0.1";
    let listen_port = 8080;
    let forward_url = Url::parse(&format!(
        "http://{}",
        ("127.0.0.1", 3000)
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
    .bind((listen_addr, listen_port))?
    .run()
}


// let matches = clap::App::new("HTTP Proxy")
//     .arg(
//         Arg::with_name("listen_addr")
//             .takes_value(true)
//             .value_name("LISTEN ADDR")
//             .index(1)
//             .required(true),
//     )
//     .arg(
//         Arg::with_name("listen_port")
//             .takes_value(true)
//             .value_name("LISTEN PORT")
//             .index(2)
//             .required(true),
//     )
//     .arg(
//         Arg::with_name("forward_addr")
//             .takes_value(true)
//             .value_name("FWD ADDR")
//             .index(3)
//             .required(true),
//     )
//     .arg(
//         Arg::with_name("forward_port")
//             .takes_value(true)
//             .value_name("FWD PORT")
//             .index(4)
//             .required(true),
//     )
//     .get_matches();

// let listen_addr = matches.value_of("listen_addr").unwrap();
// let listen_port = value_t!(matches, "listen_port", u16).unwrap_or_else(|e| e.exit());

// let forwarded_addr = matches.value_of("forward_addr").unwrap();
// let forwarded_port =
//     value_t!(matches, "forward_port", u16).unwrap_or_else(|e| e.exit());