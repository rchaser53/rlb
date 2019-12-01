use actix_web::{
    get, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};

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

pub fn server(bind_url: &str) -> std::io::Result<()> {
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
