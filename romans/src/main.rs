
static VALUE_TO_STRING : [(u32, &'static str); 17] = [
    (10000, "ↂ"), (9000, "ↁↂ"), (5000, "ↁ"), (4000, "Mↁ"),
     (1000, "M"),  (900, "CM"),  (500, "D"),  (400, "CD"),
      (100, "C"),   (90, "XC"),   (50, "L"),   (40, "XL"),
       (10, "X"),    (9, "IX"),    (5, "V"),    (4, "IV"),
        (1, "I")];

#[test]
fn value_to_string_has_only_positive_nonzero_numbers() {
    for item in VALUE_TO_STRING.iter() {
        assert_ne!(0, item.0);
    }
}

fn convert(i: u32, init: String, values: &[(u32, &str)]) -> String {
    let mut ii = i;
    let mut result = init;
    let mut matched = true;
    while ii > 0 && matched {
        matched = false;
        for item in values.iter() {
            if ii >= item.0 {
                result += item.1;
                ii -= item.0;
                matched = true;
                break;
            }
        }
    }
    result
}

#[test]
fn convert_terminates_always() {
    let bla = vec![(1000, "1km"), (50, "fsd")];
    assert_eq!("fdsa", convert(4, String::from("fdsa"), &bla));
}

#[test]
fn convert_returns_init_with_empty_slices() {
    let bla : Vec<(u32, &str)> = Vec::new();
    assert_eq!("fdsa", convert(4, String::from("fdsa"), &bla));
}

#[test]
fn convert_processes_values_in_order() {
    let bla = vec![(1000, "1km"), (5000, "fsd"), (3, "")];
    assert_eq!("fdsa1km1km1km1km1km", convert(5000, String::from("fdsa"), &bla));
}

fn to_roman(ii: u32) -> String {
    return convert(ii, String::from(""), &VALUE_TO_STRING);
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
    assert_eq!("CI", to_roman(101));
    assert_eq!("CCCXCIV", to_roman(394));
    assert_eq!("CDXCIV", to_roman(494));
    assert_eq!("DI", to_roman(501));
    assert_eq!("DCCCLXXVI", to_roman(876));
    assert_eq!("DCCCLXXXIX", to_roman(889));
    assert_eq!("CM", to_roman(900));
    assert_eq!("CMI", to_roman(901));
    assert_eq!("CMXCIX", to_roman(999));
    assert_eq!("M", to_roman(1000));
    assert_eq!("MI", to_roman(1001));
}

fn main() {
    println!("Hello, world!");
//    let value_to_string = vec![(100, "C"), (90, "XC")];
}
