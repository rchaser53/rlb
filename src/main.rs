use crossbeam_utils::thread;

mod proxy;
mod server;

use proxy::proxy;
use server::server;

fn main() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let _ = thread::scope(|scope| {
        scope.spawn(|_| server("127.0.0.1:8081"));
        scope.spawn(|_| server("127.0.0.1:8080"));
        scope.spawn(|_| proxy());
    })
    .unwrap();
}
