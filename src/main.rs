#[macro_use]
extern crate crossbeam_channel;
extern crate crossbeam_utils;

use crossbeam_channel::bounded;
use crossbeam_utils::thread;

use std::sync::mpsc;
// use std::thread;

static NTHREAD: i32 = 3;

fn main() {
  let people = vec!["Anna", "Bob", "Cody", "Dave", "Eva"];
  let (s, r) = bounded(1);  // Make room for one unmatched send.

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

// fn foo() {
//     let (tx, rx) = mpsc::channel();
//     let mut children = vec![];

//     for id in 0..NTHREAD {
//         let thread_tx = tx.clone();

//         thread::scope(|scope| {
//             let child = scope.spawn(move |_| {
//                 thread_tx.send(id).unwrap();
//                 println!("thread {} finished", id);
//             });
//             children.push(child);
//         }).unwrap();
//         // let child = thread::spawn(move || {
//         //     thread_tx.send(id).unwrap();
//         //     println!("thread {} finished", id);
//         // });
//     }

//     let mut ids = Vec::with_capacity(NTHREAD as usize);
//     for _ in 0..NTHREAD {
//         ids.push(rx.recv());
//     }

//     for child in children {
//         child.join().expect("the child thread is paniced");
//     }

//     // Show the order in whicn the messages were sent
//     println!("{:?}", ids);
// }
