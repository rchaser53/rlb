#[macro_use]
extern crate crossbeam_channel;

use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use std::env::set_var;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Backend {
    url: String,
    alive: bool,
    // mux:
    // reverse_proxy

    // URL          *url.URL
    // Alive        bool
    // mux          sync.RWMutex
    // ReverseProxy *httputil.ReverseProxy
}

#[derive(Debug)]
struct ServerPool {
    backends: Vec<Backend>,
    current: usize, // backends []*Backend
                    // current  uint64
}

// u, _ := url.Parse("http://localhost:8080")
// rp := httputil.NewSingleHostReverseProxy(u)

// // initialize your server and add this as handler
// http.HandlerFunc(rp.ServeHTTP)

fn index(_req: HttpRequest) -> &'static str {
    "Hello world"
}
fn main() {
    set_var("RUST_LOG", "sample_lb_info");
    env_logger::init();

    let _ = thread::scope(|scope| {
        scope.spawn(|_| server("127.0.0.1:8081"));
        scope.spawn(|_| server("127.0.0.1:8080"));
    })
    .unwrap();   
}

fn server(bind_url: &str) -> std::io::Result<()> {
    println!("{}", bind_url);
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/index.html").to(|| "hello"))
            .service(web::resource("/").to(index))
    })
    .bind(bind_url)?
    .run()
}

fn _aaa() {
    let people = vec!["Anna", "Bob", "Cody", "Dave", "Eva"];
    let (s, r) = bounded(1); // Make room for one unmatched send.

    let seek = |name, s, r| {
        select! {
            recv(r) -> peer => println!("{} recieved a message from {}.", name, peer.unwrap()),
            send(s, name) -> _ => {}, // Wait for someone to receive my message
        }
    };

    thread::scope(|scope| {
        for name in people {
            let (s, r) = (s.clone(), r.clone());
            scope.spawn(move |_| seek(name, s, r));
        }
    })
    .unwrap();

    if let Ok(name) = r.try_recv() {
        println!("No one recieved {}'s message.", name);
    }
}

static NTHREAD: i32 = 3;
fn _bbb() {
    let (tx, rx) = bounded(NTHREAD as usize);
    let children = Arc::new(Mutex::new(vec![]));

    for id in 0..NTHREAD {
        let thread_tx = tx.clone();
        let cloned = children.clone();
        thread::scope(|scope| {
            let _ = scope.spawn(move |_| {
                thread_tx.send(id).unwrap();
                cloned.lock().unwrap().push(id);
                println!("thread {} finished", id);
            });
        })
        .unwrap();
    }

    let mut ids = Vec::with_capacity(NTHREAD as usize);
    for _ in 0..NTHREAD {
        ids.push(rx.recv());
    }

    println!("children: {:?}", children);
    println!("ids: {:?}", ids);
}
