
static VALUE_TO_STRING : [(u32, &'static str); 9] = [
    (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
     (10, "X"),  (9, "IX"),  (5, "V"),  (4, "IV"),
      (1, "I")];

fn iterate(i: u32, init: String, values: [(u32, &str); 9]) -> String {
    let mut ii = i;
    let mut result = init;
    while ii > 0 {
        for item in values.iter() {
            if ii >= item.0 {
                result += item.1;
                ii -= item.0;
                break;
            }
        }
    }
    result
}

fn to_roman(ii: u32) -> String {
    return iterate(ii, String::from(""), VALUE_TO_STRING);
}

#[test]
fn test_to_roman() {
    assert_eq!("I", to_roman(1));
    assert_eq!("II", to_roman(2));
    assert_eq!("III", to_roman(3));
    assert_eq!("IV", to_roman(4));
    assert_eq!("V", to_roman(5));
    assert_eq!("VI", to_roman(6));
    assert_eq!("IX", to_roman(9));
    assert_eq!("X", to_roman(10));
    assert_eq!("XI", to_roman(11));
    assert_eq!("XIX", to_roman(19));
    assert_eq!("XX", to_roman(20));
    assert_eq!("XXI", to_roman(21));
    assert_eq!("XL", to_roman(40));
    assert_eq!("XLIX", to_roman(49));
    assert_eq!("L", to_roman(50));
    assert_eq!("LX", to_roman(60));
    assert_eq!("LXI", to_roman(61));
    assert_eq!("LXXXIX", to_roman(89));
    assert_eq!("XC", to_roman(90));
    assert_eq!("XCIV", to_roman(94));
    assert_eq!("XCIX", to_roman(99));
    assert_eq!("C", to_roman(100));
}

fn main() {
    println!("Hello, world!");
//    let value_to_string = vec![(100, "C"), (90, "XC")];
}
