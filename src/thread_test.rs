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
