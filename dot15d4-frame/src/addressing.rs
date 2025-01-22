//! Addressing fields readers and writers.

use super::FrameControl;
use super::FrameVersion;
use super::{Error, Result};

/// An IEEE 802.15.4 address.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Address {
    /// The address is absent.
    Absent,
    /// A short address.
    Short([u8; 2]),
    /// An extended address.
    Extended([u8; 8]),
}

impl Address {
    /// The broadcast address.
    pub const BROADCAST: Address = Address::Short([0xff; 2]);

    /// Query whether the address is an unicast address.
    pub fn is_unicast(&self) -> bool {
        !self.is_broadcast()
    }

    /// Query whether this address is the broadcast address.
    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }

    /// Create an [`Address`] from a slice of bytes.
    pub fn from(a: &[u8]) -> Self {
        if a.is_empty() {
            Address::Absent
        } else if a.len() == 2 {
            let mut b = [0u8; 2];
            b.copy_from_slice(a);
            Address::Short(b)
        } else if a.len() == 8 {
            let mut b = [0u8; 8];
            b.copy_from_slice(a);
            Address::Extended(b)
        } else {
            unreachable!()
        }
    }

    /// Return the address as a slice of bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Address::Absent => &[],
            Address::Short(value) => value,
            Address::Extended(value) => value,
        }
    }

    /// Return the short address form of the address.
    pub fn to_short(&self) -> Option<Self> {
        match self {
            short @ Address::Short(_) => Some(*short),
            Address::Extended(value) => {
                let mut raw = [0u8; 2];
                raw.copy_from_slice(&value[..2]);
                Some(Address::Short(raw))
            }
            _ => None,
        }
    }

    /// Create a short [`Address`] from an array of 2 bytes.
    const fn short_from_bytes(a: [u8; 2]) -> Self {
        Self::Short(a)
    }

    /// Create an extended [`Address`] from an array of 8 bytes.
    const fn extended_from_bytes(a: [u8; 8]) -> Self {
        Self::Extended(a)
    }

    /// Return the length of the address in octets.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Address::Absent => 0,
            Address::Short(_) => 2,
            Address::Extended(_) => 8,
        }
    }

    /// Query whether the address is absent.
    pub fn is_absent(&self) -> bool {
        matches!(self, Address::Absent)
    }

    /// Query whether the address is short.
    pub fn is_short(&self) -> bool {
        matches!(self, Address::Short(_))
    }

    /// Query whether the address is extended.
    pub fn is_extended(&self) -> bool {
        matches!(self, Address::Extended(_))
    }

    /// Parse an address from a string.
    #[cfg(any(feature = "std", test))]
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Address::Absent);
        }

        let parts: std::vec::Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => {
                let mut bytes = [0u8; 2];
                for (i, part) in parts.iter().enumerate() {
                    bytes[i] = u8::from_str_radix(part, 16).unwrap();
                }
                Ok(Address::Short(bytes))
            }
            8 => {
                let mut bytes = [0u8; 8];
                for (i, part) in parts.iter().enumerate() {
                    bytes[i] = u8::from_str_radix(part, 16).unwrap();
                }
                Ok(Address::Extended(bytes))
            }
            _ => Err(Error),
        }
    }
}

impl From<Address> for AddressingMode {
    fn from(value: Address) -> Self {
        match value {
            Address::Absent => AddressingMode::Absent,
            Address::Short(_) => AddressingMode::Short,
            Address::Extended(_) => AddressingMode::Extended,
        }
    }
}

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Address::Absent => write!(f, "absent"),
            Address::Short(value) => write!(f, "{:02x}:{:02x}", value[0], value[1]),
            Address::Extended(value) => write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7]
            ),
        }
    }
}

/// IEEE 802.15.4 addressing mode.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum AddressingMode {
    /// The address is absent.
    Absent = 0b00,
    /// The address is a short address.
    Short = 0b10,
    /// The address is an extended address.
    Extended = 0b11,
    /// Unknown addressing mode.
    Unknown,
}

impl AddressingMode {
    /// Return the size of the address in octets.
    pub fn size(&self) -> usize {
        match self {
            Self::Absent => 0,
            Self::Short => 2,
            Self::Extended => 8,
            Self::Unknown => 0,
        }
    }
}

impl From<u8> for AddressingMode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::Absent,
            0b10 => Self::Short,
            0b11 => Self::Extended,
            _ => Self::Unknown,
        }
    }
}

/// A reader/writer for the IEEE 802.15.4 Addressing Fields.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AddressingFields<T: AsRef<[u8]>, FC: AsRef<[u8]>> {
    buffer: T,
    fc: FrameControl<FC>,
}

impl<T: AsRef<[u8]>, FC: AsRef<[u8]>> AddressingFields<T, FC> {
    /// Create a new [`AddressingFields`] reader/writer from a given buffer.
    ///
    /// # Errors
    ///
    /// This function will check the length of the buffer to ensure it is large
    /// enough to contain the addressing fields. If the buffer is too small,
    /// an error will be returned.
    pub fn new(buffer: T, fc: FrameControl<FC>) -> Result<Self> {
        let af = Self::new_unchecked(buffer, fc);

        if !af.check_len() {
            return Err(Error);
        }

        Ok(af)
    }

    /// Check if the buffer is large enough to contain the addressing fields.
    fn check_len(&self) -> bool {
        let Some((dst_pan_id_present, dst_addr_mode, src_pan_id_present, src_addr_mode)) =
            Self::address_present_flags(
                self.fc.frame_version(),
                self.fc.dst_addressing_mode(),
                self.fc.src_addressing_mode(),
                self.fc.pan_id_compression(),
            )
        else {
            return false;
        };

        let expected_len = (if dst_pan_id_present { 2 } else { 0 })
            + dst_addr_mode.size()
            + (if src_pan_id_present { 2 } else { 0 })
            + src_addr_mode.size();

        self.buffer.as_ref().len() >= expected_len
    }

    /// Create a new [`AddressingFields`] reader/writer from a given buffer
    /// without checking the length.
    pub fn new_unchecked(buffer: T, fc: FrameControl<FC>) -> Self {
        Self { buffer, fc }
    }

    /// Return the length of the Addressing Fields in octets.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        (match self.dst_pan_id() {
            Some(_) => 2,
            None => 0,
        }) + match self.fc.dst_addressing_mode() {
            AddressingMode::Absent => 0,
            AddressingMode::Short => 2,
            AddressingMode::Extended => 8,
            _ => unreachable!(),
        } + match self.src_pan_id() {
            Some(_) => 2,
            None => 0,
        } + match self.fc.src_addressing_mode() {
            AddressingMode::Absent => 0,
            AddressingMode::Short => 2,
            AddressingMode::Extended => 8,
            _ => unreachable!(),
        }
    }

    fn address_present_flags(
        frame_version: FrameVersion,
        dst_addr_mode: AddressingMode,
        src_addr_mode: AddressingMode,
        pan_id_compression: bool,
    ) -> Option<(bool, AddressingMode, bool, AddressingMode)> {
        use AddressingMode::*;
        match frame_version {
            FrameVersion::Ieee802154_2003 | FrameVersion::Ieee802154_2006 => {
                match (dst_addr_mode, src_addr_mode, pan_id_compression) {
                    // If both destination and source address information is present, and the
                    // destination and source PAN IDs are identical, then the source PAN ID is
                    // omitted.
                    // In the following case, the destination and source PAN IDs are not identical,
                    // and thus both are present.
                    (dst @ (Short | Extended), src @ (Short | Extended), false) => {
                        Some((true, dst, true, src))
                    }
                    // In the following case, the destination and source PAN IDs are identical, and
                    // thus only the destination PAN ID is present.
                    (dst @ (Short | Extended), src @ (Short | Extended), true) => {
                        Some((true, dst, false, src))
                    }

                    // If either the destination or the source address is present, then the PAN ID
                    // of the corresponding address is present and the PAN ID compression field is
                    // set to 0.
                    (Absent, src @ (Short | Extended), false) => Some((false, Absent, true, src)),
                    (dst @ (Short | Extended), Absent, false) => Some((true, dst, false, Absent)),

                    // All other cases are invalid.
                    _ => None,
                }
            }
            FrameVersion::Ieee802154_2020 => {
                Some(match (dst_addr_mode, src_addr_mode, pan_id_compression) {
                    (Absent, Absent, false) => (false, Absent, false, Absent),
                    (Absent, Absent, true) => (true, Absent, false, Absent),
                    (dst, Absent, false) if !matches!(dst, Absent) => (true, dst, false, Absent),
                    (dst, Absent, true) if !matches!(dst, Absent) => (false, dst, false, Absent),
                    (Absent, src, false) if !matches!(src, Absent) => (false, Absent, true, src),
                    (Absent, src, true) if !matches!(src, Absent) => (false, Absent, false, src),
                    (Extended, Extended, false) => (true, Extended, false, Extended),
                    (Extended, Extended, true) => (false, Extended, false, Extended),
                    (Short, Short, false) => (true, Short, true, Short),
                    (Short, Extended, false) => (true, Short, true, Extended),
                    (Extended, Short, false) => (true, Extended, true, Short),
                    (Short, Extended, true) => (true, Short, false, Extended),
                    (Extended, Short, true) => (true, Extended, false, Short),
                    (Short, Short, true) => (true, Short, false, Short),
                    _ => return None,
                })
            }
            _ => None,
        }
    }

    /// Return the IEEE 802.15.4 destination [`Address`] if not absent.
    pub fn dst_address(&self) -> Option<Address> {
        if let Some((dst_pan_id, dst_addr, _, _)) = Self::address_present_flags(
            self.fc.frame_version(),
            self.fc.dst_addressing_mode(),
            self.fc.src_addressing_mode(),
            self.fc.pan_id_compression(),
        ) {
            let offset = if dst_pan_id { 2 } else { 0 };

            match dst_addr {
                AddressingMode::Absent => Some(Address::Absent),
                AddressingMode::Short => {
                    let mut raw = [0u8; 2];
                    raw.clone_from_slice(&self.buffer.as_ref()[offset..offset + 2]);
                    raw.reverse();
                    Some(Address::short_from_bytes(raw))
                }
                AddressingMode::Extended => {
                    let mut raw = [0u8; 8];
                    raw.clone_from_slice(&self.buffer.as_ref()[offset..offset + 8]);
                    raw.reverse();
                    Some(Address::extended_from_bytes(raw))
                }
                AddressingMode::Unknown => None,
            }
        } else {
            None
        }
    }

    /// Return the IEEE 802.15.4 source [`Address`] if not absent.
    pub fn src_address(&self) -> Option<Address> {
        if let Some((dst_pan_id, dst_addr, src_pan_id, src_addr)) = Self::address_present_flags(
            self.fc.frame_version(),
            self.fc.dst_addressing_mode(),
            self.fc.src_addressing_mode(),
            self.fc.pan_id_compression(),
        ) {
            let mut offset = if dst_pan_id { 2 } else { 0 };
            offset += dst_addr.size();
            offset += if src_pan_id { 2 } else { 0 };

            match src_addr {
                AddressingMode::Absent => Some(Address::Absent),
                AddressingMode::Short => {
                    let mut raw = [0u8; 2];
                    raw.clone_from_slice(&self.buffer.as_ref()[offset..offset + 2]);
                    raw.reverse();
                    Some(Address::short_from_bytes(raw))
                }
                AddressingMode::Extended => {
                    let mut raw = [0u8; 8];
                    raw.clone_from_slice(&self.buffer.as_ref()[offset..offset + 8]);
                    raw.reverse();
                    Some(Address::extended_from_bytes(raw))
                }
                AddressingMode::Unknown => None,
            }
        } else {
            None
        }
    }

    /// Return the IEEE 802.15.4 destination PAN ID if not elided.
    pub fn dst_pan_id(&self) -> Option<u16> {
        if let Some((true, _, _, _)) = Self::address_present_flags(
            self.fc.frame_version(),
            self.fc.dst_addressing_mode(),
            self.fc.src_addressing_mode(),
            self.fc.pan_id_compression(),
        ) {
            let b = &self.buffer.as_ref()[..2];
            Some(u16::from_le_bytes([b[0], b[1]]))
        } else {
            None
        }
    }

    /// Return the IEEE 802.15.4 source PAN ID if not elided.
    pub fn src_pan_id(&self) -> Option<u16> {
        if let Some((dst_pan_id, dst_addr, true, _)) = Self::address_present_flags(
            self.fc.frame_version(),
            self.fc.dst_addressing_mode(),
            self.fc.src_addressing_mode(),
            self.fc.pan_id_compression(),
        ) {
            let mut offset = if dst_pan_id { 2 } else { 0 };
            offset += dst_addr.size();

            let b = &self.buffer.as_ref()[offset..][..2];
            Some(u16::from_le_bytes([b[0], b[1]]))
        } else {
            None
        }
    }
}

impl<T: AsRef<[u8]>, FC: AsRef<[u8]>> core::fmt::Display for AddressingFields<T, FC> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Addressing Fields")?;

        if let Some(id) = self.dst_pan_id() {
            writeln!(f, "  dst pan id: {:0x}", id)?;
        }

        if let Some(addr) = self.dst_address() {
            writeln!(f, "  dst address: {}", addr)?;
        }

        if let Some(id) = self.src_pan_id() {
            writeln!(f, "  src pan id: {:0x}", id)?;
        }

        if let Some(addr) = self.src_address() {
            writeln!(f, "  src address: {}", addr)?;
        }

        Ok(())
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>, FC: AsRef<[u8]>> AddressingFields<T, FC> {
    /// Write the addressing fields to the buffer.
    pub fn write_fields(&mut self, fields: &super::repr::AddressingFieldsRepr) {
        let mut offset = 0;

        if let Some(id) = fields.dst_pan_id {
            let b = &mut self.buffer.as_mut()[offset..][..2];
            b.copy_from_slice(&id.to_le_bytes());
            offset += 2;
        }

        if let Some(addr) = fields.dst_address {
            let b = &mut self.buffer.as_mut()[offset..][..addr.len()];
            match addr {
                Address::Absent => {}
                Address::Short(value) => {
                    let mut addr = value;
                    addr.reverse();
                    b.copy_from_slice(&addr);
                }
                Address::Extended(value) => {
                    let mut addr = value;
                    addr.reverse();
                    b.copy_from_slice(&addr);
                }
            }
            offset += addr.len();
        }

        if let Some(id) = fields.src_pan_id {
            let b = &mut self.buffer.as_mut()[offset..][..2];
            b.copy_from_slice(&id.to_le_bytes());
            offset += 2;
        }

        if let Some(addr) = fields.src_address {
            let b = &mut self.buffer.as_mut()[offset..][..addr.len()];
            match addr {
                Address::Absent => {}
                Address::Short(value) => {
                    let mut addr = value;
                    addr.reverse();
                    b.copy_from_slice(&addr);
                }
                Address::Extended(value) => {
                    let mut addr = value;
                    addr.reverse();
                    b.copy_from_slice(&addr);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_type() {
        assert!(Address::Absent.is_absent());
        assert!(!Address::Absent.is_short());
        assert!(!Address::Absent.is_extended());

        assert!(!Address::Short([0xff, 0xff]).is_absent());
        assert!(Address::Short([0xff, 0xff]).is_short());
        assert!(!Address::Short([0xff, 0xff]).is_extended());

        assert!(!Address::Extended([0xff; 8]).is_absent());
        assert!(!Address::Extended([0xff; 8]).is_short());
        assert!(Address::Extended([0xff; 8]).is_extended());

        assert_eq!(Address::Absent.len(), 0);
        assert_eq!(Address::Short([0xff, 0xff]).len(), 2);
        assert_eq!(Address::Extended([0xff; 8]).len(), 8);
    }

    #[test]
    fn addressing_mode() {
        assert_eq!(AddressingMode::from(0b00), AddressingMode::Absent);
        assert_eq!(AddressingMode::from(0b10), AddressingMode::Short);
        assert_eq!(AddressingMode::from(0b11), AddressingMode::Extended);
        assert_eq!(AddressingMode::from(0b01), AddressingMode::Unknown);

        assert_eq!(AddressingMode::Unknown.size(), 0);
        assert_eq!(AddressingMode::Absent.size(), 0);
        assert_eq!(AddressingMode::Short.size(), 2);
        assert_eq!(AddressingMode::Extended.size(), 8);
    }

    #[test]
    fn is_broadcast() {
        assert!(Address::BROADCAST.is_broadcast());
        assert!(Address::Short([0xff, 0xff]).is_broadcast());
        assert!(!Address::Short([0xff, 0xfe]).is_broadcast());

        assert!(!Address::BROADCAST.is_unicast());
        assert!(!Address::Short([0xff, 0xff]).is_unicast());
        assert!(Address::Short([0xff, 0xfe]).is_unicast());
    }

    #[test]
    fn as_bytes() {
        assert_eq!(Address::BROADCAST.as_bytes(), &[0xff, 0xff]);
        assert_eq!(Address::Short([0xff, 0xff]).as_bytes(), &[0xff, 0xff]);
        assert_eq!(Address::Short([0xff, 0xfe]).as_bytes(), &[0xff, 0xfe]);
        assert_eq!(Address::Extended([0xff; 8]).as_bytes(), &[0xff; 8]);
        assert_eq!(Address::Extended([0x01; 8]).as_bytes(), &[0x01; 8]);
        assert_eq!(Address::Absent.as_bytes(), &[]);
    }

    #[test]
    fn from_bytes() {
        assert_eq!(Address::from(&[0xff, 0xff]), Address::Short([0xff, 0xff]));
        assert_eq!(Address::from(&[0xff, 0xfe]), Address::Short([0xff, 0xfe]));
        assert_eq!(Address::from(&[0xff; 8]), Address::Extended([0xff; 8]));
        assert_eq!(Address::from(&[0x01; 8]), Address::Extended([0x01; 8]));
        assert_eq!(Address::from(&[]), Address::Absent);
    }

    #[test]
    #[should_panic]
    fn from_bytes_panic() {
        Address::from(&[0xff, 0xff, 0xff]);
    }

    #[test]
    fn address_present_flags() {
        use AddressingMode::*;
        use FrameVersion::*;

        macro_rules! check {
            (($version:ident, $dst:ident, $src:ident, $compression:literal) -> $expected:expr) => {
                assert_eq!(
                    AddressingFields::<&[u8], &[u8]>::address_present_flags(
                        $version,
                        $dst,
                        $src,
                        $compression
                    ),
                    $expected
                );
            };
        }

        check!((Ieee802154_2003, Short, Short, false) -> Some((true, Short, true, Short)));
        check!((Ieee802154_2003, Short, Short, true) -> Some((true, Short, false, Short)));
        check!((Ieee802154_2003, Extended, Extended, false) -> Some((true, Extended, true, Extended)));
        check!((Ieee802154_2003, Extended, Extended, true) -> Some((true, Extended, false, Extended)));
        check!((Ieee802154_2003, Short, Extended, false) -> Some((true, Short, true, Extended)));
        check!((Ieee802154_2003, Short, Extended, true) -> Some((true, Short, false, Extended)));
        check!((Ieee802154_2003, Extended, Short, false) -> Some((true, Extended, true, Short)));
        check!((Ieee802154_2003, Extended, Short, true) -> Some((true, Extended, false, Short)));
        check!((Ieee802154_2003, Absent, Short, false) -> Some((false, Absent, true, Short)));
        check!((Ieee802154_2003, Absent, Extended, false) -> Some((false, Absent, true, Extended)));
        check!((Ieee802154_2003, Short, Absent, false) -> Some((true, Short, false, Absent)));
        check!((Ieee802154_2003, Extended, Absent, false) -> Some((true, Extended, false, Absent)));
        check!((Ieee802154_2003, Absent, Short, true) -> None);
        check!((Ieee802154_2003, Absent, Extended, true) -> None);
        check!((Ieee802154_2003, Short, Absent, true) -> None);
        check!((Ieee802154_2003, Extended, Absent, true) -> None);
        check!((Ieee802154_2003, Absent, Absent, false) -> None);
        check!((Ieee802154_2003, Absent, Absent, true) -> None);

        check!((Ieee802154_2006, Short, Short, false) -> Some((true, Short, true, Short)));
        check!((Ieee802154_2006, Short, Short, true) -> Some((true, Short, false, Short)));
        check!((Ieee802154_2006, Extended, Extended, false) -> Some((true, Extended, true, Extended)));
        check!((Ieee802154_2006, Extended, Extended, true) -> Some((true, Extended, false, Extended)));
        check!((Ieee802154_2006, Short, Extended, false) -> Some((true, Short, true, Extended)));
        check!((Ieee802154_2006, Short, Extended, true) -> Some((true, Short, false, Extended)));
        check!((Ieee802154_2006, Extended, Short, false) -> Some((true, Extended, true, Short)));
        check!((Ieee802154_2006, Extended, Short, true) -> Some((true, Extended, false, Short)));
        check!((Ieee802154_2006, Absent, Short, false) -> Some((false, Absent, true, Short)));
        check!((Ieee802154_2006, Absent, Extended, false) -> Some((false, Absent, true, Extended)));
        check!((Ieee802154_2006, Short, Absent, false) -> Some((true, Short, false, Absent)));
        check!((Ieee802154_2006, Extended, Absent, false) -> Some((true, Extended, false, Absent)));
        check!((Ieee802154_2006, Absent, Short, true) -> None);
        check!((Ieee802154_2006, Absent, Extended, true) -> None);
        check!((Ieee802154_2006, Short, Absent, true) -> None);
        check!((Ieee802154_2006, Extended, Absent, true) -> None);
        check!((Ieee802154_2006, Absent, Absent, false) -> None);
        check!((Ieee802154_2006, Absent, Absent, true) -> None);

        check!((Ieee802154_2020, Short, Short, false) -> Some((true, Short, true, Short)));
        check!((Ieee802154_2020, Short, Short, true) -> Some((true, Short, false, Short)));
        check!((Ieee802154_2020, Extended, Extended, false) -> Some((true, Extended, false, Extended)));
        check!((Ieee802154_2020, Extended, Extended, true) -> Some((false, Extended, false, Extended)));
        check!((Ieee802154_2020, Short, Extended, false) -> Some((true, Short, true, Extended)));
        check!((Ieee802154_2020, Short, Extended, true) -> Some((true, Short, false, Extended)));
        check!((Ieee802154_2020, Extended, Short, false) -> Some((true, Extended, true, Short)));
        check!((Ieee802154_2020, Extended, Short, true) -> Some((true, Extended, false, Short)));
        check!((Ieee802154_2020, Absent, Short, false) -> Some((false, Absent, true, Short)));
        check!((Ieee802154_2020, Absent, Extended, false) -> Some((false, Absent, true, Extended)));
        check!((Ieee802154_2020, Short, Absent, false) -> Some((true, Short, false, Absent)));
        check!((Ieee802154_2020, Extended, Absent, false) -> Some((true, Extended, false, Absent)));
        check!((Ieee802154_2020, Absent, Short, true) -> Some((false, Absent, false, Short)));
        check!((Ieee802154_2020, Absent, Extended, true) -> Some((false, Absent, false, Extended)));
        check!((Ieee802154_2020, Short, Absent, true) -> Some((false, Short, false, Absent)));
        check!((Ieee802154_2020, Extended, Absent, true) -> Some((false, Extended, false, Absent)));
        check!((Ieee802154_2020, Absent, Absent, false) -> Some((false, Absent, false, Absent)));
        check!((Ieee802154_2020, Absent, Absent, true) -> Some((true, Absent, false, Absent)));
    }

    #[test]
    fn parse() {
        let mut addresses = vec![
            ("", Address::Absent),
            ("ff:ff", Address::Short([0xff, 0xff])),
            ("ff:fe", Address::Short([0xff, 0xfe])),
            ("ff:ff:ff:ff:ff:ff:ff:ff", Address::Extended([0xff; 8])),
            ("01:01:01:01:01:01:01:01", Address::Extended([0x01; 8])),
            ("00:00:00:00:00:00:00:00", Address::Extended([0x00; 8])),
            (
                "00:00:00:00:00:00:00:01",
                Address::Extended([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
            ),
        ];

        for (s, expected) in addresses.drain(..) {
            assert_eq!(Address::parse(s).unwrap(), expected);
        }
    }
}
