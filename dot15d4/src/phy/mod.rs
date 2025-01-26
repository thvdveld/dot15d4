//! Access to IEEE 802.15.4 devices.
//!
//! This module provides access to IEEE 802.15.4 devices. It provides a trait
//! for transmitting and recieving frames, [Device].

pub mod config;
pub mod constants;
pub mod radio;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// Ack failed, after to many retransmissions
    AckFailed,
    /// The buffer did not follow the correct device structure
    InvalidDeviceStructure,
    /// Something went wrong in the radio
    RadioError,
}

/// A buffer that is used to store 1 frame.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBuffer {
    /// The data of the frame that should be transmitted over the radio.
    /// Normally, a MAC layer frame is 127 bytes long.
    /// Some radios, like the one on the nRF52840, have the possibility to not
    /// having to include the checksum at the end, but require one extra byte to
    /// specify the length of the frame. This results in this case in
    /// needing 126 bytes in total. However, some radios like the one on the
    /// Zolertia Zoul, add extra information about the link quality. In that
    /// case, the total buffer length for receiving data comes on 128 bytes.
    /// If you would like to support a radio that needs more than 128 bytes,
    /// please file a PR.
    pub buffer: [u8; 128],
    /// Whether or not the buffer is ready to be read from
    pub dirty: bool,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; 128],
            dirty: false,
        }
    }
}
