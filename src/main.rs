#[macro_use]
extern crate crossbeam_channel;

use crossbeam_channel::bounded;
use crossbeam_utils::thread;
use std::sync::{Arc, Mutex};

fn bar() {
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
fn main() {
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
