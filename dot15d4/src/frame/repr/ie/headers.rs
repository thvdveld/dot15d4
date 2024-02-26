pub use super::super::super::{HeaderElementId, HeaderInformationElement, TimeCorrection};

use crate::time::Duration;

/// A high-level representation of a Header Information Element.
#[derive(Debug)]
pub enum HeaderInformationElementRepr {
    TimeCorrection(TimeCorrectionRepr),
    HeaderTermination1,
    HeaderTermination2,
}

impl HeaderInformationElementRepr {
    pub fn parse(ie: HeaderInformationElement<&[u8]>) -> Self {
        match ie.element_id() {
            HeaderElementId::TimeCorrection => Self::TimeCorrection(TimeCorrectionRepr {
                time_correction: TimeCorrection::new(ie.content()).time_correction(),
                nack: TimeCorrection::new(ie.content()).nack(),
            }),
            HeaderElementId::HeaderTermination1 => Self::HeaderTermination1,
            HeaderElementId::HeaderTermination2 => Self::HeaderTermination2,
            element => todo!("Received {element:?}"),
        }
    }
}

/// A high-level representation of a Time Correction Header Information Element.
#[derive(Debug)]
pub struct TimeCorrectionRepr {
    /// The time correction value in microseconds.
    pub time_correction: Duration,
    /// The negative acknowledgment flag.
    pub nack: bool,
}
