use core::convert::TryFrom;

use crate::Cep18Error;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum EventsMode {
    NoEvents = 0,
    CES = 1,
    Native = 2,
    NativeNCES = 3,
}

impl TryFrom<u8> for EventsMode {
    type Error = Cep18Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventsMode::NoEvents),
            1 => Ok(EventsMode::CES),
            2 => Ok(EventsMode::Native),
            3 => Ok(EventsMode::NativeNCES),
            _ => Err(Cep18Error::InvalidEventsMode),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum MintBurn {
    Disabled = 0,
    MintAndBurn = 1,
}

impl TryFrom<u8> for MintBurn {
    type Error = Cep18Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MintBurn::Disabled),
            1 => Ok(MintBurn::MintAndBurn),
            _ => Err(Cep18Error::InvalidEnableMBFlag),
        }
    }
}
