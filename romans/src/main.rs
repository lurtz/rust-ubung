
fn convert(i: u32, init: &str, values: fn(u32) -> Option<(u32, String)>) -> String {
    let mut ii = i;
    let mut result = init.to_string();
    while let Some((new_i, appendix)) = values(ii) {
        ii = new_i;
        result += appendix.as_str();
    }
    result
}

static VALUE_TO_STRING : [(u32, &'static str); 17] = [
    (10000, "ↂ"), (9000, "ↁↂ"), (5000, "ↁ"), (4000, "Mↁ"),
     (1000, "M"),  (900, "CM"),  (500, "D"),  (400, "CD"),
      (100, "C"),   (90, "XC"),   (50, "L"),   (40, "XL"),
       (10, "X"),    (9, "IX"),    (5, "V"),    (4, "IV"),
        (1, "I")];

static DIST_TO_STRING : [(u32, &'static str); 4] = [
    (1000000, "km"), (1000, "m"), (10, "cm"), (1, "mm")
];

#[test]
fn positive_nonzero_numbers() {
    for item in VALUE_TO_STRING.iter() {
        assert_ne!(0, item.0);
    }
    for item in DIST_TO_STRING.iter() {
        assert_ne!(0, item.0);
    }
}

fn gen_impl<OP0, OP1>(i: u32, values: &[(u32, &str)], ops: (OP0, OP1)) -> Option<(u32, String)> where OP0: Fn(u32, u32) -> u32, OP1: Fn(u32, u32, String) -> String {
    for item in values.iter() {
        if i >= item.0 {
            let next_i = ops.0(i, item.0);
            let string_i = ops.1(i, item.0, String::from(item.1));
            return Some((next_i, string_i))
        }
    }
    None
}

fn roman_impl(i: u32) -> Option<(u32, String)> {
    let ops = (|i: u32, itemval: u32| i - itemval,
               |_: u32, _: u32, stringval: String| stringval);
    gen_impl(i, &VALUE_TO_STRING, ops)
}

fn to_roman(ii: u32) -> String {
    return convert(ii, "", roman_impl);
}

fn dist_impl(i: u32) -> Option<(u32, String)> {
    let ops = (|i: u32, itemval: u32| i % itemval,
               |i: u32, itemval: u32, stringval: String| (i / itemval).to_string() + stringval.as_str() + " ");
    gen_impl(i, &DIST_TO_STRING, ops)
}

fn with_distance_units(i: u32) -> String {
    let result = convert(i, "", dist_impl);
    if result.len() == 0 {
        String::from("0m")
    } else {
        String::from(result.trim_right())
    }
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
