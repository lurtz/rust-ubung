use std::cmp::{Eq, PartialEq};
use std::fmt::{Display, Error, Formatter, Write};
use std::hash::{Hash, Hasher};
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
pub enum PowerState {
    On,
    Standby,
}

impl Display for PowerState {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        let mut buffer = String::new();
        write!(&mut buffer, "{:?}", self)?;
        write!(format, "{}", buffer.to_ascii_uppercase())
    }
}

impl PowerState {
    pub fn iterator() -> Iter<'static, PowerState> {
        static STATES: [PowerState; 2] = [PowerState::On, PowerState::Standby];
        STATES.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceInputState {
    Cd,
    Tuner,
    Dvd,
    Bd,
    Tv,
    Satcbl,
    Game,
    Game2,
    Vaux,
    Dock,
    Ipod,
    Netusb,
    Rhapsody,
    Napster,
    Pandora,
    Lastfm,
    Flickr,
    Favorites,
    Iradio,
    Server,
    Usbipod,
    Usb,
    Ipd,
    Irp,
    Fvp,
    Unknown,
}

impl Display for SourceInputState {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        let mut buffer = String::new();
        write!(&mut buffer, "{:?}", self)?;
        write!(format, "{}", buffer.to_ascii_uppercase())
    }
}

impl SourceInputState {
    pub fn iterator() -> Iter<'static, SourceInputState> {
        static STATES: [SourceInputState; 25] = [
            SourceInputState::Cd,
            SourceInputState::Tuner,
            SourceInputState::Dvd,
            SourceInputState::Bd,
            SourceInputState::Tv,
            SourceInputState::Satcbl,
            SourceInputState::Game,
            SourceInputState::Game2,
            SourceInputState::Vaux,
            SourceInputState::Dock,
            SourceInputState::Ipod,
            SourceInputState::Netusb,
            SourceInputState::Rhapsody,
            SourceInputState::Napster,
            SourceInputState::Pandora,
            SourceInputState::Lastfm,
            SourceInputState::Flickr,
            SourceInputState::Favorites,
            SourceInputState::Iradio,
            SourceInputState::Server,
            SourceInputState::Usbipod,
            SourceInputState::Usb,
            SourceInputState::Ipd,
            SourceInputState::Irp,
            SourceInputState::Fvp,
        ];
        STATES.iter()
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Power(PowerState),
    SourceInput(SourceInputState),
    MaxVolume(u32),
    MainVolume(u32),
    Unknown,
}

impl State {
    pub fn value(&self) -> &'static str {
        match *self {
            State::Power(_) => "PW",
            State::SourceInput(_) => "SI",
            State::MaxVolume(_) => "MVMAX",
            State::MainVolume(_) => "MV",
            State::Unknown => "Unknown",
        }
    }

    pub fn power() -> State {
        State::Power(PowerState::On)
    }

    pub fn source_input() -> State {
        State::SourceInput(SourceInputState::Dvd)
    }

    pub fn max_volume() -> State {
        State::MaxVolume(0)
    }

    pub fn main_volume() -> State {
        State::MainVolume(0)
    }
}

impl Display for State {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        match *self {
            State::Power(ref p) => write!(format, "{}{}", self.value(), p),
            State::SourceInput(ref si) => write!(format, "{}{}", self.value(), si),
            State::MaxVolume(i) => write!(format, "{}{}", self.value(), i),
            State::MainVolume(i) => write!(format, "{}{}", self.value(), i),
            State::Unknown => write!(format, "{}", self.value()),
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            State::Power(_) => 1.hash(state),
            State::SourceInput(_) => 2.hash(state),
            State::MaxVolume(_) => 3.hash(state),
            State::MainVolume(_) => 4.hash(state),
            State::Unknown => 5.hash(state),
        }
    }
}

macro_rules! equal_helper {
    ($first:ident, $second:ident, $enum_value:path) => {
        if let $enum_value(_) = *$first {
            if let $enum_value(_) = *$second {
                return true;
            }
        }
    };
}

macro_rules! equal_helper_no_args {
    ($first:ident, $second:ident, $enum_value:path) => {
        if let $enum_value = *$first {
            if let $enum_value = *$second {
                return true;
            }
        }
    };
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        equal_helper!(self, other, State::Power);
        equal_helper!(self, other, State::SourceInput);
        equal_helper!(self, other, State::MaxVolume);
        equal_helper!(self, other, State::MainVolume);
        equal_helper_no_args!(self, other, State::Unknown);
        false
    }
}

impl Eq for State {}

#[cfg(test)]
mod test {
    use crate::state::{PowerState, SourceInputState, State};
    use std::collections::HashSet;

    fn check_value(hs: &HashSet<State>, expected: &State) {
        match *expected {
            State::MainVolume(v) => {
                let get_value = State::MainVolume(v + 1);
                let value = hs.get(&get_value).unwrap();
                if let &State::MainVolume(vv) = value {
                    assert_eq!(v, vv);
                } else {
                    assert!(false);
                }
            }
            State::MaxVolume(v) => {
                let get_value = State::MaxVolume(v + 1);
                let value = hs.get(&get_value).unwrap();
                if let &State::MaxVolume(vv) = value {
                    assert_eq!(vv, v);
                } else {
                    assert!(false);
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn state_works_in_set() {
        let mut hs = HashSet::new();
        let mv_100 = State::MainVolume(100);
        hs.replace(mv_100.clone());
        assert_eq!(1, hs.len());
        check_value(&hs, &mv_100);

        let mv_129 = State::MainVolume(129);
        hs.replace(mv_129.clone());
        assert_eq!(1, hs.len());
        check_value(&hs, &mv_129);

        let maxv_100 = State::MaxVolume(100);
        hs.replace(maxv_100.clone());
        assert_eq!(2, hs.len());
        check_value(&hs, &mv_129);
        check_value(&hs, &maxv_100);
    }

    #[test]
    fn power_state_display() {
        assert_eq!("ON", PowerState::On.to_string());
        assert_eq!("STANDBY", PowerState::Standby.to_string());
    }

    #[test]
    fn source_input_state_display() {
        assert_eq!("DVD", SourceInputState::Dvd.to_string());
        assert_eq!("FLICKR", SourceInputState::Flickr.to_string());
    }
}
