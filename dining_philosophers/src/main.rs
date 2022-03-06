use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Philosopher {
    name: String,
    left: usize,
    right: usize,
}

impl Philosopher {
    fn new(name: &str, left: usize, right: usize) -> Philosopher {
        Philosopher {
            name: name.to_string(),
            left,
            right,
        }
    }

    fn eat(&self, table: &Table) {
        let _left = table.forks[self.left].lock().unwrap();
        let _right = table.forks[self.right].lock().unwrap();

        println!("{} starts eating", self.name);

        thread::sleep(Duration::from_millis(1000));

        println!("{} is done eating", self.name);
    }
}

struct Table {
    forks: Vec<Mutex<()>>,
}

fn main() {
    let name: String = "bla".to_string();

    let table = Arc::new(Table {
        forks: vec![
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
        ],
    });

    let philosophers = vec![
        Philosopher::new("Lutz", 0, 1),
        Philosopher::new("Debra", 1, 2),
        Philosopher::new("Julian", 2, 3),
        Philosopher::new("Jan", 3, 4),
        Philosopher::new("Sebastian", 4, 5),
        Philosopher::new(&name, 0, 5),
    ];

    let handles: Vec<_> = philosophers
        .into_iter()
        .map(|p| {
            let table = table.clone();
            thread::spawn(move || p.eat(&table))
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
