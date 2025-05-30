use std::fmt;

trait TraitWithAssocType {
    type X: fmt::Display;
    type Y: fmt::Debug;

    fn get_x(&self) -> Self::X;
    fn get_y(&self) -> Self::Y;
}

impl<X: fmt::Display, Y: fmt::Debug> fmt::Display for dyn TraitWithAssocType<X = X, Y = Y> {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "{}, {:?}", self.get_x(), self.get_y())
    }
}

#[derive(Debug, Clone)]
struct TestStruct {
    i: u32,
    data: Vec<u32>,
}

impl TraitWithAssocType for TestStruct {
    type X = u32;
    type Y = Vec<u32>;

    fn get_x(&self) -> Self::X {
        self.i
    }

    fn get_y(&self) -> Self::Y {
        self.data.clone()
    }
}

struct TestStruct2 {
    s: String,
    i: u32,
    data: Vec<u32>,
}

impl TraitWithAssocType for TestStruct2 {
    type X = String;
    type Y = TestStruct;

    fn get_x(&self) -> Self::X {
        self.s.clone()
    }

    fn get_y(&self) -> Self::Y {
        TestStruct {
            i: self.i,
            data: self.data.clone(),
        }
    }
}

fn bla<T: TraitWithAssocType>(x: &T) {
    println!("x is {}", x.get_x());
    println!("y is {:?}", x.get_y());
}

fn bla2<T: TraitWithAssocType<X = u32>>(x: &T) {
    println!("new x is {}", x.get_x() + 3);
    println!("y is {:?}", x.get_y());
}

fn main() {
    println!("Hello, world!");

    let x = TestStruct {
        i: 34,
        data: vec![1, 2, 3, 4, 5],
    };
    println!("{}", &x as &dyn TraitWithAssocType<X = u32, Y = Vec<u32>>);
    let y = TestStruct2 {
        s: String::from("bblabla"),
        i: 666,
        data: vec![7, 7, 7, 4, 3, 2],
    };
    println!(
        "{}",
        &y as &dyn TraitWithAssocType<X = String, Y = TestStruct>
    );

    bla(&x);
    bla(&y);

    bla2(&x);
    // bla2(&y);
}

#[cfg(test)]
mod test {
    use crate::{TestStruct, TestStruct2, TraitWithAssocType, main};

    #[test]
    fn display_returns_expected_strings() {
        let x = TestStruct {
            i: 66,
            data: vec![34, 2, 432, 432],
        };
        assert_eq!(
            "66, [34, 2, 432, 432]",
            format!("{}", &x as &dyn TraitWithAssocType<X = u32, Y = Vec<u32>>)
        );
        let y = TestStruct2 {
            s: String::from("test"),
            i: 190,
            data: vec![1, 2, 3],
        };
        assert_eq!(
            "test, TestStruct { i: 190, data: [1, 2, 3] }",
            format!(
                "{}",
                &y as &dyn TraitWithAssocType<X = String, Y = TestStruct>
            )
        );
    }

    #[test]
    fn call_main() {
        main();
    }
}
