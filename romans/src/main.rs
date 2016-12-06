
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

fn roman_impl(i: u32) -> Option<(u32, String)> {
    for item in VALUE_TO_STRING.iter() {
        if i >= item.0 {
            return Some((i - item.0, String::from(item.1)))
        }
    }

    None
}

fn convert(i: u32, init: String, values: fn(u32) -> Option<(u32, String)>) -> String {
    let mut ii = i;
    let mut result = init;
    while let Some((new_i, appendix)) = values(ii) {
        ii = new_i;
        result += appendix.as_str();
    }
    result
}

fn to_roman(ii: u32) -> String {
    return convert(ii, String::from(""), roman_impl);
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

fn dist_impl(i: u32) -> Option<(u32, String)> {
    if i >= 1000000 {
        let new_i = i/1000000;
        return Some((i % 1000000, new_i.to_string() + "km "))
    }
    if i >= 1000 {
        let new_i = i/1000;
        return Some((i % 1000, new_i.to_string() + "m "))
    }
    if i >= 10 {
        let new_i = i/10;
        return Some((i % 10, new_i.to_string() + "cm "))
    }
    if i > 0 {
        return Some((0, i.to_string() + "mm "))
    }
    None
}

fn with_distance_units(i: u32) -> String {
    let result = convert(i, String::from(""), dist_impl);
    if result.len() == 0 {
        String::from("0m")
    } else {
        let strlen = result.len();
        String::from(&result[..strlen-1])
    }
}

#[test]
fn test_with_distance_units() {
    assert_eq!("0m", with_distance_units(0));
    assert_eq!("1mm", with_distance_units(1));
    assert_eq!("1cm", with_distance_units(10));
    assert_eq!("1m", with_distance_units(1000));
    assert_eq!("1km", with_distance_units(1000000));
    assert_eq!("20km 345m 32cm 7mm", with_distance_units(20345327));
}

fn main() {
    println!("Hello, world!");
//    let value_to_string = vec![(100, "C"), (90, "XC")];
}
