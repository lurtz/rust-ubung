pub use crate::operation::Operation;
pub use crate::state::PowerState;
pub use crate::state::SourceInputState;
pub use crate::state::State;

macro_rules! parsehelper {
    ($trimmed:expr, $op:expr, $func:path) => {
        if $trimmed.starts_with($op.value()) {
            let value = get_value($trimmed, &$op);
            let x = $func(value);
            return Some(x);
        }
    };
}

fn get_value<'a>(trimmed: &'a str, op: &State) -> &'a str {
    let to_skip = op.value().len();
    trimmed[to_skip..].trim()
}

fn parse_int(to_parse: &str) -> u32 {
    let mut value = to_parse.parse::<u32>().unwrap();
    if value < 100 {
        value *= 10;
    }
    value
}

fn parse_main_volume(value: &str) -> State {
    let value = parse_int(value);
    State::MainVolume(value)
}

fn parse_max_volume(value: &str) -> State {
    let value = parse_int(value);
    State::MaxVolume(value)
}

fn parse_power(value: &str) -> State {
    if "ON" == value {
        State::Power(PowerState::On)
    } else {
        State::Power(PowerState::Standby)
    }
}

fn parse_source_input(value: &str) -> State {
    for sis in SourceInputState::iterator() {
        if sis.to_string() == value {
            return State::SourceInput(sis.clone());
        }
    }

    State::SourceInput(SourceInputState::Unknown)
}

pub fn parse(str: &str) -> Option<State> {
    let trimmed = str.trim().trim_matches('\r');
    parsehelper!(trimmed, State::max_volume(), parse_max_volume);
    parsehelper!(trimmed, State::main_volume(), parse_main_volume);
    parsehelper!(trimmed, State::power(), parse_power);
    parsehelper!(trimmed, State::source_input(), parse_source_input);
    None
}

#[cfg(test)]
mod test {
    use super::parse;
    use crate::parse::{PowerState, SourceInputState, State};

    #[test]
    fn parse_with_unknown_string() {
        assert_eq!(None, parse(""));
        assert_eq!(None, parse("blub"));
    }

    #[test]
    #[should_panic]
    fn max_volume_without_value_panics() {
        parse("MVMAX");
    }

    #[test]
    fn max_volume() {
        assert!(matches!(parse("MVMAX0"), Some(State::MaxVolume(0))));
        assert!(matches!(parse("MVMAX23"), Some(State::MaxVolume(230))));
        assert!(matches!(parse("MVMAX99"), Some(State::MaxVolume(990))));
        assert!(matches!(parse("MVMAX100"), Some(State::MaxVolume(100))));
        assert!(matches!(parse("MVMAX230"), Some(State::MaxVolume(230))));
        assert!(matches!(parse("MVMAX999"), Some(State::MaxVolume(999))));
        assert!(matches!(parse("MVMAX 999"), Some(State::MaxVolume(999))));
    }

    #[test]
    #[should_panic]
    fn main_voule_without_value_panics() {
        parse("MV");
    }

    #[test]
    fn main_voule() {
        assert!(matches!(parse("MV 0"), Some(State::MainVolume(0))));
        assert!(matches!(parse("MV 23"), Some(State::MainVolume(230))));
        assert!(matches!(parse("MV 99"), Some(State::MainVolume(990))));
        assert!(matches!(parse("MV 100"), Some(State::MainVolume(100))));
        assert!(matches!(parse("MV 230"), Some(State::MainVolume(230))));
        assert!(matches!(parse("MV 999"), Some(State::MainVolume(999))));
        assert!(matches!(parse("MV999"), Some(State::MainVolume(999))));
    }

    #[test]
    fn power() {
        assert!(matches!(
            parse("PW"),
            Some(State::Power(PowerState::Standby))
        ));
        assert!(matches!(
            parse("PWOFF"),
            Some(State::Power(PowerState::Standby))
        ));
        assert!(matches!(parse("PWON"), Some(State::Power(PowerState::On))));
    }

    #[test]
    fn source_input() {
        assert!(matches!(
            parse("SI"),
            Some(State::SourceInput(SourceInputState::Unknown))
        ));
        assert!(matches!(
            parse("SIblub"),
            Some(State::SourceInput(SourceInputState::Unknown))
        ));
        assert!(matches!(
            parse("SITV"),
            Some(State::SourceInput(SourceInputState::Tv))
        ));
    }
}
