use actix_web::client::Client;
use actix_web::{
    middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use std::net::ToSocketAddrs;
use url::Url;

pub async fn forward(
  req: HttpRequest,
  body: web::Bytes,
  url: web::Data<Url>,
  client: web::Data<Client>,
) -> HttpResponse {
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

  forwarded_req
      .send_body(body)
      .await
      .map(|res| async {
          let mut client_resp = HttpResponse::build(res.status());
          // Remove `Connection` as per
          // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
          for (header_name, header_value) in
              res.headers().iter().filter(|(h, _)| *h != "connection")
          {
              client_resp.header(header_name.clone(), header_value.clone());
          }

          client_resp.streaming(res)
      })
      .unwrap()
      .await
  // .flatten()
}

pub fn proxy() -> std::io::Result<()> {
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
  .workers(1)
  .run()
}
