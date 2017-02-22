extern crate std;
use std::fmt::{Display, Formatter, Error};
use std::slice::Iter;

#[derive(Debug,Clone)]
pub enum PowerState {
    ON,
    STANDBY,
}

impl Display for PowerState {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        match self {
            &PowerState::ON => write!(format, "ON"),
            &PowerState::STANDBY => write!(format, "STANDBY"),
        }
    }
}

#[derive(Debug,Clone)]
pub enum SourceInputState {
    CD,
    Tuner,
    DVD,
    BD,
    TV,
    SATCBL,
    GAME,
    GAME2,
    VAUX,
    DOCK,
    IPOD,
    NETUSB,
    RHAPSODY,
    NAPSTER,
    PANDORA,
    LASTFM,
    FLICKR,
    FAVORITES,
    IRADIO,
    SERVER,
    USBIPOD,
    USB,
    IPD,
    IRP,
    FVP,
    UNKNOWN,
}

impl Display for SourceInputState {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        write!(format, "{:?}", self)
    }
}

impl SourceInputState {
    pub fn iterator() -> Iter<'static, SourceInputState> {
        static STATES: [SourceInputState; 25] = [SourceInputState::CD,
                                                 SourceInputState::Tuner,
                                                 SourceInputState::DVD,
                                                 SourceInputState::BD,
                                                 SourceInputState::TV,
                                                 SourceInputState::SATCBL,
                                                 SourceInputState::GAME,
                                                 SourceInputState::GAME2,
                                                 SourceInputState::VAUX,
                                                 SourceInputState::DOCK,
                                                 SourceInputState::IPOD,
                                                 SourceInputState::NETUSB,
                                                 SourceInputState::RHAPSODY,
                                                 SourceInputState::NAPSTER,
                                                 SourceInputState::PANDORA,
                                                 SourceInputState::LASTFM,
                                                 SourceInputState::FLICKR,
                                                 SourceInputState::FAVORITES,
                                                 SourceInputState::IRADIO,
                                                 SourceInputState::SERVER,
                                                 SourceInputState::USBIPOD,
                                                 SourceInputState::USB,
                                                 SourceInputState::IPD,
                                                 SourceInputState::IRP,
                                                 SourceInputState::FVP];
        STATES.into_iter()
    }
}

#[derive(Debug,Clone)]
pub enum State {
    Integer(u32),
    String(String),
    Power(PowerState),
    SourceInput(SourceInputState),
    MaxVolume(u32),
    MainVolume(u32),
}

impl Display for State {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        match self {
            &State::Integer(i) => write!(format, "{}", i),
            &State::String(ref s) => write!(format, "{}", s),
            &State::Power(ref p) => write!(format, "{}", p),
            &State::SourceInput(ref si) => write!(format, "{}", si),
            &State::MaxVolume(i) => write!(format, "{}", i),
            &State::MainVolume(i) => write!(format, "{}", i),
        }
    }
}
