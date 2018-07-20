#![feature(macro_rules)]

extern crate core;

use core::fmt::Show;
use core::fmt::Formatter;
use core::fmt::Result;

struct Matrix<T> {
  rows: uint,
  cols: uint,
  vals: Vec<T>,
}

impl<T: NumCast + Clone> Matrix<T> {
  fn new(rows: uint, cols: uint) -> Matrix<T> {
    let init_val: T = NumCast::from(0).unwrap();
    Matrix{rows: rows, cols: cols, vals: Vec::from_fn(rows * cols, |ignore| init_val.clone())}
  }
  fn set(& mut self, row: uint, col: uint, val: T) {
    assert!(row < self.rows);
    assert!(col < self.cols);
    *self.vals.get_mut(row * self.cols + col) = val;
  }
}

impl<T: Show> Show for Matrix<T> {
  fn fmt(&self, args: &mut Formatter) -> Result {
    write!(args, "{}x{}-Matrix({})", self.rows, self.cols, self.vals)
  }
}

macro_rules! mmatrix(($($($item:expr),+);*) => ({
  let mut rows: uint = 0;
  let mut items: uint = 0;
  {
    let incrows = |ignore| {rows += 1};
    let incitems = |ignore| {items += 1};
    $( incrows(($(incitems($item),)+)); )*
  }
  let cols = items/rows;
  Matrix{rows: rows, cols: cols, vals: vec!($($($item,)+)*)}
}))

fn main() {
  let mut m: Matrix<int> = Matrix::new(4,4);
  m.set(1,1,3);
  println!("{}", m);
  let m2 = mmatrix!(1,2,3;4,5,6;7,8,9);
  println!("m2: {}", m2);
}
