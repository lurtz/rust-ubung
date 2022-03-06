use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    rows: u32,
    cols: u32,
    data: Vec<T>,
}

impl<T: Clone + Default> Matrix<T> {
    pub fn new(rows: u32, cols: u32) -> Matrix<T> {
        let init = T::default();
        Matrix {
            rows,
            cols,
            data: vec![init; (rows * cols) as usize],
        }
    }
}

impl<T> Matrix<T> {
    pub fn set(&mut self, row: u32, col: u32, val: T) {
        assert!(row < self.rows);
        assert!(col < self.cols);
        self.data[(row * self.cols + col) as usize] = val;
    }

    fn get(&self, row: u32, col: u32) -> &T {
        assert!(row < self.rows);
        assert!(col < self.cols);
        &self.data[(row * self.cols + col) as usize]
    }
}

impl<T: Debug> Display for Matrix<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}x{}-Matrix({:?})", self.rows, self.cols, self.data)
    }
}

impl<T: Clone + Mul<Output = T> + Add<Output = T>> Mul for Matrix<T> {
    type Output = Matrix<T>;

    fn mul(self, _rhs: Matrix<T>) -> Matrix<T> {
        assert!(self.cols == _rhs.rows);
        let mut data: Vec<T> = Vec::with_capacity((self.rows * _rhs.cols) as usize);

        for row in 0..self.rows {
            for col in 0.._rhs.cols {
                let mut new_val = self.get(row, 0).clone() * _rhs.get(0, col).clone();

                for ii in 1..self.cols {
                    let mult_output = self.get(row, ii).clone() * _rhs.get(ii, col).clone();
                    new_val = new_val + mult_output;
                }

                data.push(new_val);
            }
        }

        Matrix {
            rows: self.rows,
            cols: _rhs.cols,
            data,
        }
    }
}

impl<T: PartialEq> PartialEq for Matrix<T> {
    fn eq(&self, other: &Matrix<T>) -> bool {
        if self.rows != other.rows {
            return false;
        }
        if self.cols != other.cols {
            return false;
        }
        for row in 0..self.rows {
            for col in 0..self.cols {
                if *self.get(row, col) != *other.get(row, col) {
                    return false;
                }
            }
        }
        true
    }
}

impl<T: PartialEq> Eq for Matrix<T> {}

#[macro_export]
macro_rules! mmatrix(($($($item:expr),+);*) => ({
  let mut rows: u32 = 0;
  let mut items: u32 = 0;
  {
    let mut incrows = |_ignore| {rows += 1};
    let mut incitems = |_ignore| {items += 1};
    $( incrows(($(incitems($item),)+)); )*
  }
  let cols = items/rows;
  Matrix{rows: rows, cols: cols, data: vec![$($($item,)+)*]}
}));

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn new() {
        let m = Matrix::new(3, 4);
        assert_eq!(3, m.rows);
        assert_eq!(4, m.cols);
        for i in 0..3 {
            for j in 0..4 {
                assert_eq!(0, *m.get(i, j));
            }
        }
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn get_range_test_rows() {
        let m: Matrix<i32> = Matrix::new(4, 5);
        m.get(5, 0);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn get_range_test_cols() {
        let m: Matrix<i32> = Matrix::new(4, 5);
        m.get(0, 6);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn set_range_test_rows() {
        let mut m: Matrix<i32> = Matrix::new(4, 5);
        m.set(5, 0, 0);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn set_range_test_cols() {
        let mut m: Matrix<i32> = Matrix::new(4, 5);
        m.set(0, 6, 0);
    }

    #[test]
    fn set_get() {
        let mut m: Matrix<i32> = Matrix::new(4, 4);
        m.set(0, 0, 3);
        assert_eq![3, *m.get(0, 0)];
        m.set(1, 1, 6);
        assert_eq![6, *m.get(1, 1)];
        m.set(2, 2, 8);
        assert_eq![8, *m.get(2, 2)];
        m.set(3, 3, -10);
        assert_eq![-10, *m.get(3, 3)];
        m.set(0, 3, -100);
        assert_eq![-100, *m.get(0, 3)];
        m.set(3, 0, -400);
        assert_eq![-400, *m.get(3, 0)];
    }

    #[test]
    fn mmatrix() {
        //    let m = mmatrix![];
        //    assert_eq![0, m.rows];
        //    assert_eq![0, m.cols];

        let m1 = mmatrix![3];
        assert_eq![1, m1.rows];
        assert_eq![1, m1.cols];
        assert_eq![3, *m1.get(0, 0)];

        let m2 = mmatrix![1, 2, 3];
        assert_eq![1, m2.rows];
        assert_eq![3, m2.cols];
        assert_eq![1, *m2.get(0, 0)];
        assert_eq![2, *m2.get(0, 1)];
        assert_eq![3, *m2.get(0, 2)];

        let m3 = mmatrix![1;2;3];
        assert_eq![3, m3.rows];
        assert_eq![1, m3.cols];
        assert_eq![1, *m3.get(0, 0)];
        assert_eq![2, *m3.get(1, 0)];
        assert_eq![3, *m3.get(2, 0)];

        let m4 = mmatrix![1,2,3;4,5,6];
        assert_eq![2, m4.rows];
        assert_eq![3, m4.cols];
        assert_eq![1, *m4.get(0, 0)];
        assert_eq![2, *m4.get(0, 1)];
        assert_eq![3, *m4.get(0, 2)];
        assert_eq![4, *m4.get(1, 0)];
        assert_eq![5, *m4.get(1, 1)];
        assert_eq![6, *m4.get(1, 2)];
    }

    #[test]
    fn mmatrix_brackets() {
        let m = mmatrix!(1,2,3;4,5,6;7,8,9);
        let mm = mmatrix![1,2,3;4,5,6;7,8,9];
        assert_eq!(m, mm);
    }

    #[test]
    fn eq() {
        let m0 = mmatrix!(1, 2);
        let m1 = mmatrix!(2, 3);
        let m2 = mmatrix!(2;3);
        let m3 = mmatrix!(1,3,3;4,7,7);
        let m4 = mmatrix!(2,3,3;4,7,7);
        assert_eq!(m0, m0);
        assert_eq!(m1, m1);
        assert_eq!(m2, m2);
        assert_eq!(m3, m3);
        assert_eq!(m4, m4);
        assert!(m0 != m1);
        assert!(m0 != m2);
        assert!(m0 != m3);
        assert!(m0 != m4);
        assert!(m1 != m2);
        assert!(m1 != m3);
        assert!(m1 != m4);
        assert!(m2 != m3);
        assert!(m2 != m4);
        assert!(m3 != m4);
    }

    #[test]
    fn scale() {
        let m0 = mmatrix!(1,2;3,4);
        let m1 = mmatrix!(2,0;0,2);
        let result = m0 * m1;
        assert_eq!(mmatrix![2,4;6,8], result);
    }

    #[test]
    fn mult_4x4_matrix() {
        assert_eq!(
            mmatrix![19, 22; 43, 50],
            mmatrix!(1,2;3,4) * mmatrix!(5,6;7,8)
        );
    }

    #[test]
    fn select_vector_from_matrix() {
        assert_eq!(mmatrix![1;3], mmatrix!(1,2;3,4) * mmatrix!(1;0));
    }

    #[test]
    fn mult_float() {
        let m0 = mmatrix![2.0, 3.0; 4.0, 5.0];
        assert_eq!(mmatrix![16.0, 21.0; 28.0, 37.0], m0.clone() * m0);
    }
}

#[cfg(not(test))]
fn main() {}
