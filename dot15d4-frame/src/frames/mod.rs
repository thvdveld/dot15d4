//! High-level representation of IEEE 802.15.4 frames.

use crate::{Error, Result};

use crate::{AddressingFields, AuxiliarySecurityHeader, FrameControl, FrameType, FrameVersion};

pub(crate) mod ack;
pub(crate) mod beacon;
pub(crate) mod data;

pub use ack::*;
pub use beacon::*;
pub use data::*;

/// A high-level representation of an IEEE 802.15.4 frame with a Frame Check Sequence (FCS).
pub struct FrameWithFcs<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> FrameWithFcs<T> {
    /// Create a new [`FrameWithFcs`] from a given buffer.
    pub fn new(buffer: T) -> Result<Self> {
        let mut frame = Self::new_unchecked(buffer);

        if !frame.check_len() {
            return Err(Error);
        }

        if !frame.check_fcs() {
            return Err(Error);
        }

        Ok(frame)
    }

    /// Check the length of the frame.
    pub fn check_len(&self) -> bool {
        if self.buffer.as_ref().len() < 2 {
            return false;
        }

        true
    }

    /// Check the Frame Check Sequence (FCS) of the frame.
    pub fn check_fcs(&self) -> bool {
        // The FCS field contains a 16-bit ITU-T CRC, which is
        // calculated over the entire frame, excluding the FCS field itself.
        // The CRC-16-IBM/SDLC polynomial is used.
        let crc = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        crc.checksum(self.content()) == self.fcs()
    }

    /// Create a new [`FrameWithFcs`] from a given buffer without checking the FCS.
    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    /// Return the content of the frame, excluding the FCS.
    pub fn content(&self) -> &[u8] {
        &self.buffer.as_ref()[..self.buffer.as_ref().len() - 2]
    }

    /// Return the Frame Check Sequence (FCS) of the frame.
    pub fn fcs(&self) -> u16 {
        let len = self.buffer.as_ref().len();
        u16::from_le_bytes([self.buffer.as_ref()[len - 2], self.buffer.as_ref()[len - 1]])
    }

    /// Return a high-level representation of the frame, excluding the FCS.
    pub fn frame(&self) -> Result<Frame<&'_ [u8]>> {
        Frame::new(self.content())
    }
}

/// A high-level representation of an IEEE 802.15.4 frame.
pub enum Frame<T: AsRef<[u8]>> {
    /// An acknowledgment frame.
    Ack(Ack<T>),
    /// An enhanced acknowledgment frame.
    EnhancedAck(EnhancedAck<T>),
    /// A beacon frame.
    Beacon(Beacon<T>),
    /// An enhanced beacon frame.
    EnhancedBeacon(EnhancedBeacon<T>),
    /// A data frame.
    Data(DataFrame<T>),
}

impl<T: AsRef<[u8]>> Frame<T> {
    /// Create a new [`Frame`] from a given buffer.
    pub fn new(buffer: T) -> Result<Self> {
        if buffer.as_ref().len() < 2 {
            return Err(Error);
        }

        let frame_control = FrameControl::new(&buffer.as_ref()[..2])?;

        match frame_control.frame_type() {
            FrameType::Ack => match frame_control.frame_version() {
                FrameVersion::Ieee802154_2003 | FrameVersion::Ieee802154_2006 => {
                    Ok(Frame::Ack(Ack::new(buffer)?))
                }
                FrameVersion::Ieee802154_2020 => Ok(Frame::EnhancedAck(EnhancedAck::new(buffer)?)),
                FrameVersion::Unknown => Err(Error),
            },
            FrameType::Beacon => match frame_control.frame_version() {
                FrameVersion::Ieee802154_2003 | FrameVersion::Ieee802154_2006 => {
                    Ok(Frame::Beacon(Beacon::new(buffer)?))
                }
                FrameVersion::Ieee802154_2020 => {
                    Ok(Frame::EnhancedBeacon(EnhancedBeacon::new(buffer)?))
                }
                FrameVersion::Unknown => Err(Error),
            },
            FrameType::Data => Ok(Frame::Data(DataFrame::new(buffer)?)),
            _ => Err(Error),
        }
    }

    /// Convert the [`Frame`] into an [`Ack`].
    ///
    /// # Panics
    /// Panics if the frame is not an ack.
    pub fn into_ack(self) -> Ack<T> {
        match self {
            Frame::Ack(frame) => frame,
            _ => panic!("not an ack"),
        }
    }

    /// Convert the [`Frame`] into an [`EnhancedAck`].
    ///
    /// # Panics
    /// Panics if the frame is not an enhanced ack.
    pub fn into_enhanced_ack(self) -> EnhancedAck<T> {
        match self {
            Frame::EnhancedAck(frame) => frame,
            _ => panic!("not an enhanced ack"),
        }
    }

    /// Convert the [`Frame`] into a [`Beacon`].
    ///
    /// # Panics
    /// Panics if the frame is not a beacon frame.
    pub fn into_beacon(self) -> Beacon<T> {
        match self {
            Frame::Beacon(frame) => frame,
            _ => panic!("not a beacon"),
        }
    }

    /// Convert the [`Frame`] into an [`EnhancedBeacon`].
    ///
    /// # Panics
    /// Panics if the frame is not an enhanced beacon frame.
    pub fn into_enhanced_beacon(self) -> EnhancedBeacon<T> {
        match self {
            Frame::EnhancedBeacon(frame) => frame,
            _ => panic!("not an enhanced beacon"),
        }
    }

    /// Convert the [`Frame`] into a [`DataFrame`].
    ///
    /// # Panics
    /// Panics if the frame is not a data frame.
    pub fn into_data(self) -> DataFrame<T> {
        match self {
            Frame::Data(frame) => frame,
            _ => panic!("not a data frame"),
        }
    }

    /// Return the frame control field of the frame.
    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        match self {
            Frame::Ack(frame) => frame.frame_control(),
            Frame::EnhancedAck(frame) => frame.frame_control(),
            Frame::Beacon(frame) => frame.frame_control(),
            Frame::EnhancedBeacon(frame) => frame.frame_control(),
            Frame::Data(frame) => frame.frame_control(),
        }
    }

    /// Return the sequence number of the frame.
    pub fn sequence_number(&self) -> Option<u8> {
        match self {
            Frame::Ack(frame) => Some(frame.sequence_number()),
            Frame::EnhancedAck(frame) => frame.sequence_number(),
            Frame::Beacon(frame) => Some(frame.sequence_number()),
            Frame::EnhancedBeacon(frame) => frame.sequence_number(),
            Frame::Data(frame) => frame.sequence_number(),
        }
    }

    /// Return the addressing field of the frame.
    pub fn addressing(&self) -> Option<AddressingFields<&'_ [u8], &'_ [u8]>> {
        match self {
            Frame::Ack(frame) => None,
            Frame::EnhancedAck(frame) => frame.addressing(),
            Frame::Beacon(frame) => Some(frame.addressing()),
            Frame::EnhancedBeacon(frame) => frame.addressing(),
            Frame::Data(frame) => frame.addressing(),
        }
    }

    /// Return the auxiliary security header of the frame.
    pub fn auxiliary_security_header(&self) -> Option<AuxiliarySecurityHeader<&'_ [u8]>> {
        match self {
            Frame::Ack(_) => None,
            Frame::EnhancedAck(frame) => frame.auxiliary_security_header(),
            Frame::Beacon(frame) => frame.auxiliary_security_header(),
            Frame::EnhancedBeacon(frame) => frame.auxiliary_security_header(),
            Frame::Data(frame) => frame.auxiliary_security_header(),
        }
    }

    /// Return the information elements of the frame.
    pub fn information_elements(&self) -> Option<crate::InformationElements<&'_ [u8]>> {
        match self {
            Frame::Ack(_) => None,
            Frame::EnhancedAck(frame) => frame.information_elements(),
            Frame::Beacon(frame) => None,
            Frame::EnhancedBeacon(frame) => frame.information_elements(),
            Frame::Data(frame) => frame.information_elements(),
        }
    }
}

impl<T: AsRef<[u8]> + ?Sized> Frame<&'_ T> {
    /// Return the payload of the frame.
    pub fn payload(&self) -> Option<&[u8]> {
        match self {
            Frame::Ack(frame) => None,
            Frame::EnhancedAck(frame) => frame.payload(),
            Frame::Beacon(frame) => frame.payload(),
            Frame::EnhancedBeacon(frame) => frame.payload(),
            Frame::Data(frame) => frame.payload(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(missing_docs)]
    macro_rules! test {
        (
            $data:expr, $expected:pat, $into:ident
        ) => {{
            let data = hex::decode($data).unwrap();
            let frame = Frame::new(data).unwrap();
            assert!(matches!(frame, $expected));
            frame.$into()
        }};
    }

    #[test]
    fn high_level_parsing() {
        test!("021001", Frame::Ack(_), into_ack);
        test!(
            "022e37cdab02000200020002000200020fe18f",
            Frame::EnhancedAck(_),
            into_enhanced_ack
        );
        test!(
            "40ebcdabffff0100010001000100003f1188061a0e0000000000011c0001c800011b00",
            Frame::EnhancedBeacon(_),
            into_enhanced_beacon
        );
        test!(
            "41d801cdabffffc7d9b514004b12002b000000",
            Frame::Data(_),
            into_data
        );
    }
}
