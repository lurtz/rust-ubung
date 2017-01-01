macro_rules! my_vec {
    () => ( Vec::new() );
    ( $( $x:expr ),+ ) => ({ let mut v = Vec::new(); $(v.push($x);)+ v });
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_brackets() {
        let x = my_vec!(1,2,3);
        assert_eq!(3, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
    }

    #[test]
    fn square_brackets() {
        let x = my_vec![1,2,3];
        assert_eq!(3, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
    }

    #[test]
    fn empty_vector() {
        let x : Vec<u32> = my_vec![];
        assert_eq!(0, x.len());
    }

    #[test]
    fn one_element() {
        let x = my_vec![5];
        assert_eq!(1, x.len());
        assert_eq!(5, x[0]);
    }

    #[test]
    fn vector_with_content() {
        let x = my_vec![1,2,3,4,5];
        assert_eq!(5, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
        assert_eq!(4, x[3]);
        assert_eq!(5, x[4]);
    }
}
