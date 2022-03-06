trait DoSomething {
    fn doit(&self);
}

struct Doer {
    x: i32,
}

impl Doer {
    fn new() -> Doer {
        Doer { x: 0 }
    }
}

impl DoSomething for Doer {
    fn doit(&self) {
        println!("did it with {}!", self.x);
    }
}

fn bla(x: &Doer) {
    x.doit();
}

fn main() {
    let x = "5".parse::<i32>();
    println!("{:?}", x);

    match x {
        Ok(x) => println!("got {}", x),
        Err(_) => unreachable!(),
    }

    let x = Doer::new();
    x.doit();
    bla(&x);
    //  x.bla();
}
