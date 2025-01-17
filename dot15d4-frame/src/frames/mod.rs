//! High-level representation of IEEE 802.15.4 frames.

use crate::{Error, Result};

use crate::{AddressingFields, AuxiliarySecurityHeader, FrameControl, FrameType, FrameVersion};

pub(crate) mod ack;
pub(crate) mod beacon;
pub(crate) mod data;

pub use ack::*;
pub use beacon::*;
pub use data::*;

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
