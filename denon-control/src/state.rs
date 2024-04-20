use std::cmp::{Eq, PartialEq};
use std::fmt::{Display, Error, Formatter, Write};
use std::hash::{Hash, Hasher};
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        match *self {
            SourceInputState::Netusb => write!(&mut buffer, "NET/USB")?,
            SourceInputState::Usbipod => write!(&mut buffer, "USB/IPOD")?,
            _ => write!(&mut buffer, "{:?}", self)?,
        }
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
        if matches!(*$first, $enum_value(_)) && matches!(*$second, $enum_value(_)) {
            return true;
        }
    };
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        equal_helper!(self, other, State::Power);
        equal_helper!(self, other, State::SourceInput);
        equal_helper!(self, other, State::MaxVolume);
        equal_helper!(self, other, State::MainVolume);
        matches!(self, State::Unknown) && matches!(other, State::Unknown)
    }
}

impl Eq for State {}

#[cfg(test)]
mod test {
    use crate::state::{PowerState, SourceInputState, State};
    use std::collections::HashSet;

    fn check_value(hs: &HashSet<State>, expected: &State) {
        match expected {
            State::MainVolume(v) => {
                let get_value = State::MainVolume(v + 1);
                let value = hs.get(&get_value).unwrap();
                assert!(matches!(value, State::MainVolume(vv) if vv == v));
            }
            State::MaxVolume(v) => {
                let get_value = State::MaxVolume(v + 1);
                let value = hs.get(&get_value).unwrap();
                assert!(matches!(value, State::MaxVolume(vv) if vv == v));
            }
            State::Power(p) => {
                let get_value = match p {
                    PowerState::On => State::Power(PowerState::Standby),
                    PowerState::Standby => State::Power(PowerState::On),
                };
                let value = hs.get(&get_value).unwrap();
                assert!(matches!(value, State::Power(vv) if vv == p));
            }
            State::SourceInput(si) => {
                let get_value = State::SourceInput(SourceInputState::Ipd);
                let value = hs.get(&get_value).unwrap();
                assert!(matches!(value, State::SourceInput(vv) if vv == si));
            }
            State::Unknown => {
                let value = hs.get(&State::Unknown).unwrap();
                assert!(matches!(value, State::Unknown));
            }
        }
    }

    #[test]
    fn state_equal_main_volume() {
        // TODO maybe this behavior is not the best one. == should also compare the stored integer
        assert_eq!(State::MainVolume(12), State::MainVolume(23));
        assert_ne!(State::MainVolume(12), State::MaxVolume(23));
        assert_ne!(State::MainVolume(12), State::MaxVolume(12));
        assert_ne!(State::MainVolume(12), State::Power(PowerState::On));
        assert_ne!(
            State::MainVolume(12),
            State::SourceInput(SourceInputState::Bd)
        );
        assert_ne!(State::MainVolume(12), State::Unknown);
    }

    #[test]
    fn state_equal_max_volume() {
        assert_eq!(State::MaxVolume(12), State::MaxVolume(23));
        assert_ne!(State::MaxVolume(12), State::MainVolume(23));
        assert_ne!(State::MaxVolume(12), State::MainVolume(12));
        assert_ne!(State::MaxVolume(12), State::Power(PowerState::On));
        assert_ne!(
            State::MaxVolume(12),
            State::SourceInput(SourceInputState::Bd)
        );
        assert_ne!(State::MaxVolume(12), State::Unknown);
    }

    #[test]
    fn state_equal_power() {
        assert_eq!(State::Power(PowerState::On), State::Power(PowerState::On));
        assert_eq!(
            State::Power(PowerState::On),
            State::Power(PowerState::Standby)
        );
        assert_ne!(State::Power(PowerState::On), State::MainVolume(23));
        assert_ne!(State::Power(PowerState::On), State::MaxVolume(12));
        assert_ne!(
            State::Power(PowerState::On),
            State::SourceInput(SourceInputState::Bd)
        );
        assert_ne!(State::Power(PowerState::On), State::Unknown);
    }

    #[test]
    fn state_equal_source_input() {
        assert_eq!(
            State::SourceInput(SourceInputState::Bd),
            State::SourceInput(SourceInputState::Cd)
        );
        assert_eq!(
            State::SourceInput(SourceInputState::Bd),
            State::SourceInput(SourceInputState::Fvp)
        );
        assert_ne!(
            State::SourceInput(SourceInputState::Bd),
            State::MainVolume(23)
        );
        assert_ne!(
            State::SourceInput(SourceInputState::Bd),
            State::MaxVolume(12)
        );
        assert_ne!(
            State::SourceInput(SourceInputState::Bd),
            State::Power(PowerState::On)
        );
        assert_ne!(State::SourceInput(SourceInputState::Bd), State::Unknown);
    }

    #[test]
    fn state_equal_unknown() {
        assert_eq!(State::Unknown, State::Unknown);
        assert_ne!(State::Unknown, State::MainVolume(23));
        assert_ne!(State::Unknown, State::MaxVolume(12));
        assert_ne!(State::Unknown, State::SourceInput(SourceInputState::Bd));
        assert_ne!(State::Unknown, State::SourceInput(SourceInputState::Bd));
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

        let power_on = State::Power(PowerState::On);
        hs.replace(power_on.clone());
        assert_eq!(3, hs.len());
        check_value(&hs, &mv_129);
        check_value(&hs, &maxv_100);
        check_value(&hs, &power_on);

        let sibd = State::SourceInput(SourceInputState::Bd);
        hs.replace(sibd.clone());
        assert_eq!(4, hs.len());
        check_value(&hs, &mv_129);
        check_value(&hs, &maxv_100);
        check_value(&hs, &power_on);
        check_value(&hs, &sibd);

        let unkown = State::Unknown;
        hs.replace(unkown.clone());
        assert_eq!(5, hs.len());
        check_value(&hs, &mv_129);
        check_value(&hs, &maxv_100);
        check_value(&hs, &power_on);
        check_value(&hs, &sibd);
        check_value(&hs, &unkown);
    }

    #[test]
    fn power_state_iterator() {
        let mut piter = PowerState::iterator();
        assert!(matches!(piter.next(), Some(PowerState::On)));
        assert!(matches!(piter.next(), Some(PowerState::Standby)));
        assert!(matches!(piter.next(), None));
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

    #[test]
    fn state_display() {
        assert_eq!("MV23", State::MainVolume(23).to_string());
        assert_eq!("MVMAX230", State::MaxVolume(230).to_string());
        assert_eq!("PWON", State::Power(PowerState::On).to_string());
        assert_eq!(
            "SIDOCK",
            State::SourceInput(SourceInputState::Dock).to_string()
        );
        assert_eq!("Unknown", State::Unknown.to_string());
    }
}
