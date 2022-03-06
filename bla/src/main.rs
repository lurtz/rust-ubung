use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello, world!");

    let x = (1..111).collect::<Vec<_>>();
    println!("{:?}", x);

    for i in 30..50 {
        let greater_than_42 = (1..i).find(|x| *x > 42);

        println!("{:?}", greater_than_42);

        match greater_than_42 {
            Some(x) => println!("{}", x),
            None => println!("None"),
        }
    }

    println!("{}", (1..20).fold(10, |sum, x| sum + x));

    let y = (2..20).map(|x| x * x);
    let z = y.fold(1, |sum, x| x / sum);
    println!("{}", z);

    for i in (1..).take(10) {
        println!("{}", i);
    }

    let t = (1..)
        .map(|x| 2 * x)
        .filter(|&x| x % 3 == 0)
        .filter(|&x| x < 100)
        .take(5)
        .collect::<Vec<_>>();
    println!("{:?}", t);

    let xx = 3 as f32 * 2.0;
    println!("{}", xx);

    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let (tx, rx) = mpsc::channel();

    for i in 0..3 {
        let (data, tx) = (data.clone(), tx.clone());
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(500));
            let mut data = data.lock().unwrap();
            data[i] += 1;
            match tx.send(42) {
                Ok(x) => println!("was able to send and got {:?}", x),
                Err(x) => println!("was not able to send and got {}", x),
            }
        });
    }

    for _ in 0..3 {
        println!("{}", rx.recv().ok().expect("fail at receive of 42"));
    }

    println!("{:?}", *data.lock().unwrap());

    let panic_thread = thread::spawn(|| {
        panic!("rums");
    })
    .join();
    match panic_thread {
        Ok(x) => println!("thread started with {:?}", x),
        Err(x) => println!("thread crashed with {:?}", x),
    }
}
