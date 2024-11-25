//! The `adder` crate provides functions that add numbers to other numbers.
//!
//! # Examples
//!
//! ```
//! assert_eq!(3.0, adder::bla(3.0, 2.0, 0.0));
//! ```

/// This function adds two to its argument.
///
/// # Examples
///
/// ```
/// use adder::bla;
///
/// assert_eq!(3.0, bla(3.0, 2.0, 0.0));
/// ```
pub fn bla(x: f32, y: f32, z: f32) -> f32 {
    ((x * y) + z) / (y + z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(0.0, bla(0.0, 1.0, 0.0));
    }

    #[test]
    fn it_works2() {
        assert_eq!(1.0, bla(0.0, 0.0, 1.0));
    }

    #[test]
    fn it_tests() {
        let x = (1..111).collect::<Vec<_>>();
        println!("{:?}", x);
    }
}
