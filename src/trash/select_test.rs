

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