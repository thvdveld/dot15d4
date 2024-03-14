//! Auxiliary Security Header readers and writers.

// TODO: once this part is finished, remove the `allow` directive.
#![allow(missing_docs)]

/// A reader/writer for the IEEE 802.15.4 Auxiliary Security Header.
#[derive(Debug)]
pub struct AuxiliarySecurityHeader<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> AuxiliarySecurityHeader<T> {
    pub fn new(buffer: T) -> Self {
        Self { buffer }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        1 + self.security_control().security_level().mic_length()
    }

    pub fn security_control(&self) -> SecurityControl {
        SecurityControl::from(self.buffer.as_ref()[0])
    }
}

/// A reader/writer for the IEEE 802.15.4 Security Control field.
pub struct SecurityControl {
    buffer: u8,
}

impl SecurityControl {
    pub fn from(buffer: u8) -> Self {
        Self { buffer }
    }

    /// Return the security level field.
    pub fn security_level(&self) -> SecurityLevel {
        SecurityLevel::from(self.buffer & 0b111)
    }

    /// Return the key identifier mode field.
    pub fn key_identifier_mode(&self) -> KeyIdentifierField {
        KeyIdentifierField::from((self.buffer >> 3) & 0b11)
    }

    /// Returns `true` the frame counter is suppressed.
    pub fn frame_counter_suppression(&self) -> bool {
        (self.buffer >> 5) & 0b1 == 1
    }

    /// Returns `true` when the ASN is included in the nonce.
    pub fn asn_in_nonce(&self) -> bool {
        (self.buffer >> 6) & 0b1 == 1
    }
}

/// A Security Level field.
pub struct SecurityLevel {
    buffer: u8,
}

impl SecurityLevel {
    pub fn from(buffer: u8) -> Self {
        Self { buffer }
    }

    /// Return the used Security Attributes.
    pub fn security_attributes(&self) -> SecurityAttributes {
        match self.buffer {
            0 => SecurityAttributes::None,
            1 => SecurityAttributes::Mic32,
            2 => SecurityAttributes::Mic64,
            3 => SecurityAttributes::Mic128,
            5 => SecurityAttributes::EncMic32,
            6 => SecurityAttributes::EncMic64,
            7 => SecurityAttributes::EncMic128,
            _ => SecurityAttributes::Unknown,
        }
    }

    /// Return `true` when confidentiality is enabled.
    pub fn data_confidentiality(&self) -> bool {
        (self.buffer >> 3) & 0b1 == 1
    }

    /// Return `true` when authenticity is enabled.
    pub fn data_authenticity(&self) -> bool {
        self.buffer != 0
    }

    /// Return the MIC length.
    pub fn mic_length(&self) -> usize {
        match self.security_attributes() {
            SecurityAttributes::Mic32 | SecurityAttributes::EncMic32 => 4,
            SecurityAttributes::Mic64 | SecurityAttributes::EncMic64 => 8,
            SecurityAttributes::Mic128 | SecurityAttributes::EncMic128 => 16,
            _ => 0,
        }
    }
}

pub enum SecurityAttributes {
    None,
    Mic32,
    Mic64,
    Mic128,
    EncMic32,
    EncMic64,
    EncMic128,
    Unknown,
}

/// A Key Identifier Mode field.
pub struct KeyIdentifierField {
    buffer: u8,
}

impl KeyIdentifierField {
    pub fn from(buffer: u8) -> Self {
        Self { buffer }
    }

    /// Return the Key Identifier Mode.
    pub fn key_identifier_mode(&self) -> KeyIdentifierMode {
        match self.buffer {
            0 => KeyIdentifierMode::Implicit,
            1 => KeyIdentifierMode::Explicit,
            _ => KeyIdentifierMode::Unknown,
        }
    }
}

pub enum KeyIdentifierMode {
    Implicit,
    Explicit,
    Unknown,
}
