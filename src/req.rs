use actix_web::client::{Client, ClientRequest};
use actix_web::{
    dev, http::Uri, web
};
use std::net::ToSocketAddrs;
use url::Url;

pub fn create_forwarded_req(
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

pub fn create_forward_url(original_url: &Uri, host: &str, port: u16) -> Url {
  let mut new_url = Url::parse(&format!(
      "http://{}",
      (host, port).to_socket_addrs().unwrap().next().unwrap()
  ))
  .unwrap();

  new_url.set_path(original_url.path());
  new_url.set_query(original_url.query());
  new_url
}