use std::sync::mpsc;
use std::thread;

static NTHREAD: i32 = 3;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut children = vec![];

    for id in 0..NTHREAD {
        let thread_tx = tx.clone();

        let child = thread::spawn(move || {
            thread_tx.send(id).unwrap();
            println!("thread {} finished", id);
        });

        children.push(child);
    }

    let mut ids = Vec::with_capacity(NTHREAD as usize);
    for _ in 0..NTHREAD {
        ids.push(rx.recv());
    }

    for child in children {
        child.join().expect("the child thread is paniced");
    }

    // Show the order in whicn the messages were sent
    println!("{:?}", ids);
}
