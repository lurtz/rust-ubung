#![feature(core)]
#![feature(num)]

extern crate core;
extern crate num;

use core::fmt::Display;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result;

use num;

struct Matrix<T> {
  rows: u32,
  cols: u32,
  vals: Vec<T>,
}

impl<T: Clone + NumCast> Matrix<T> {
  fn new(rows: u32, cols: u32) -> Matrix<T> {
    let mut vec: Vec<T> = Vec::new();
    vec.resize((rows*cols) as usize, NumCast::from(0).unwrap());
    Matrix{rows: rows, cols: cols, vals: vec}
  }

  fn set(& mut self, m: u32, n: u32, val: T) {}
}

impl<T: Debug> Debug for Matrix<T> {
  fn fmt(&self, args: &mut Formatter) -> Result {
    write!(args, "{}x{}-Matrix({:?})", self.rows, self.cols, self.vals)
  }
}

fn print_vec<T: Debug>(vec: & Vec<T>) {
  println!("vec: {:?}", vec);
}

fn main() {
/*
  let mut m: Matrix<int> = Matrix::new(4,4);
  m.set(1,1,3);
  println!("{}", m);
  let m2 = mmatrix!(1,2,3;4,5,6;7,8,9);
  println!("m2: {}", m2);
*/

  let mut m: Matrix<u16> = Matrix::new(4,4);
  println!("m: {:?}", m);

  let mut vals: Vec<u32> = Vec::new();
  vals.push(1);
  vals.push(3);
  vals.push(9);
  println!("vals: {:?}", vals);
  print_vec(&vals);
}
