#[macro_export]
macro_rules! my_vec {
    () => ( Vec::new() );
    ( $( $x:expr ),+ ) => ({ let mut v = Vec::new(); $(v.push($x);)+ v });
}

pub fn skip_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    if line.starts_with(prefix) {
        let pref_len: usize = prefix.len();
        return &line[pref_len..];
    }
    line
}

#[cfg(test)]
mod tests {
    use crate::skip_prefix;

    #[test]
    fn round_brackets() {
        let x = my_vec!(1, 2, 3);
        assert_eq!(3, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
    }

    #[test]
    fn square_brackets() {
        let x = my_vec![1, 2, 3];
        assert_eq!(3, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
    }

    #[test]
    fn empty_vector() {
        let x: Vec<u32> = my_vec![];
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
        let x = my_vec![1, 2, 3, 4, 5];
        assert_eq!(5, x.len());
        assert_eq!(1, x[0]);
        assert_eq!(2, x[1]);
        assert_eq!(3, x[2]);
        assert_eq!(4, x[3]);
        assert_eq!(5, x[4]);
    }

    #[test]
    fn scopes_und_so() {
        let line = String::from("lang:en=Hello World!");
        let lang = "en";

        let p = format!("lang:{}=", lang);
        let x = skip_prefix(line.as_str(), p.as_str());
        assert_eq!("Hello World!", x);
        let y = skip_prefix(line.as_str(), "blub");
        assert_eq!(y, y)
    }
}
