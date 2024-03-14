use super::super::super::{Error, Result};
use super::super::super::{HeaderElementId, HeaderInformationElement, TimeCorrection};

use crate::time::Duration;

/// A high-level representation of a Header Information Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum HeaderInformationElementRepr {
    /// Time Correction Header Information Element.
    TimeCorrection(TimeCorrectionRepr),
    /// Header Termination 1.
    HeaderTermination1,
    /// Header Termination 2.
    HeaderTermination2,
}

impl HeaderInformationElementRepr {
    /// Parse a Header Information Element.
    pub fn parse(ie: &HeaderInformationElement<&[u8]>) -> Result<Self> {
        Ok(match ie.element_id() {
            HeaderElementId::TimeCorrection => Self::TimeCorrection(TimeCorrectionRepr::parse(
                &TimeCorrection::new(ie.content())?,
            )),
            HeaderElementId::HeaderTermination1 => Self::HeaderTermination1,
            HeaderElementId::HeaderTermination2 => Self::HeaderTermination2,
            _ => return Err(Error),
        })
    }

    /// The buffer length required to emit the Header Information Element.
    pub fn buffer_len(&self) -> usize {
        2 + self.inner_len()
    }

    /// The buffer length required to emit the inner part of the Header
    /// Information Element.
    fn inner_len(&self) -> usize {
        match self {
            Self::TimeCorrection(tc) => tc.buffer_len(),
            Self::HeaderTermination1 => 0,
            Self::HeaderTermination2 => 0,
        }
    }

    /// Emit the Header Information Element into a buffer.
    pub fn emit(&self, buffer: &mut [u8]) {
        let mut w = HeaderInformationElement::new_unchecked(&mut buffer[..]);
        w.clear();
        w.set_length(self.inner_len() as u16);
        w.set_element_id(self.into());

        match self {
            Self::TimeCorrection(repr) => {
                repr.emit(&mut TimeCorrection::new_unchecked(w.content_mut()));
            }
            Self::HeaderTermination1 => {}
            Self::HeaderTermination2 => {}
        }
    }
}

impl From<&HeaderInformationElementRepr> for HeaderElementId {
    fn from(val: &HeaderInformationElementRepr) -> Self {
        use HeaderInformationElementRepr::*;
        match val {
            TimeCorrection(_) => HeaderElementId::TimeCorrection,
            HeaderTermination1 => HeaderElementId::HeaderTermination1,
            HeaderTermination2 => HeaderElementId::HeaderTermination2,
        }
    }
}

/// A high-level representation of a Time Correction Header Information Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TimeCorrectionRepr {
    /// The time correction value in microseconds.
    pub time_correction: Duration,
    /// The negative acknowledgment flag.
    pub nack: bool,
}

impl TimeCorrectionRepr {
    /// Parse a Time Correction Header Information Element.
    pub fn parse(tc: &TimeCorrection<&'_ [u8]>) -> Self {
        Self {
            time_correction: tc.time_correction(),
            nack: tc.nack(),
        }
    }

    /// The buffer length required to emit the Time Correction Header
    /// Information Element.
    pub const fn buffer_len(&self) -> usize {
        2
    }

    /// Emit the Time Correction Header Information Element into a buffer.
    pub fn emit(&self, buffer: &mut TimeCorrection<&mut [u8]>) {
        buffer.set_time_correction(self.time_correction);
        buffer.set_nack(self.nack);
    }
}
