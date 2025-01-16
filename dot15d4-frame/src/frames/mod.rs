use crate::{Error, Result};

use crate::{FrameControl, FrameType, FrameVersion};

pub(crate) mod ack;
pub(crate) mod beacon;
pub(crate) mod data;

pub use ack::Ack;
pub use ack::EnhancedAck;
pub use beacon::Beacon;
pub use beacon::EnhancedBeacon;
pub use data::DataFrame;

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

    /// Convert the [`Frame`] into a [`BeaconFrame`].
    ///
    /// # Panics
    /// Panics if the frame is not a beacon frame.
    pub fn into_beacon(self) -> Beacon<T> {
        match self {
            Frame::Beacon(frame) => frame,
            _ => panic!("not a beacon"),
        }
    }

    /// Convert the [`Frame`] into an [`EnhancedBeaconFrame`].
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
}
