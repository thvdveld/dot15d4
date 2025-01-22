use super::NestedInformationElementsIterator;
use super::{Error, Result};

/// A reader/writer for the IEEE 802.15.4 Payload Information Elements.
#[derive(Debug, Eq, PartialEq)]
pub struct PayloadInformationElement<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> PayloadInformationElement<T> {
    /// Create a new [`PayloadInformationElement`] reader/writer from a given
    /// buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short to contain a payload
    /// information element.
    pub fn new(data: T) -> Result<Self> {
        let ie = Self::new_unchecked(data);

        if !ie.check_len() {
            return Err(Error);
        }

        Ok(ie)
    }

    /// Returns `false` if the buffer is too short to contain a payload
    /// information element.
    fn check_len(&self) -> bool {
        self.data.as_ref().len() >= 2
    }

    /// Create a new [`PayloadInformationElement`] reader/writer from a given
    /// buffer without length checking.
    pub fn new_unchecked(data: T) -> Self {
        Self { data }
    }

    /// Return the length field value (which is the lenght of the content field).
    pub fn length(&self) -> usize {
        let b = &self.data.as_ref()[0..2];
        u16::from_le_bytes([b[0], b[1]]) as usize & 0b1111111111
    }

    /// Return the [`PayloadGroupId`].
    pub fn group_id(&self) -> PayloadGroupId {
        let b = &self.data.as_ref()[0..2];
        let id = (u16::from_le_bytes([b[0], b[1]]) >> 11) & 0b111;
        PayloadGroupId::from(id as u8)
    }

    /// Return the content of this Header Information Element.
    pub fn content(&self) -> &[u8] {
        &self.data.as_ref()[2..][..self.length()]
    }

    /// Returns [`NestedInformationElementsIterator`] [`Iterator`].
    ///
    /// ## Panics
    /// This method panics if the [`PayloadInformationElement`] is not an
    /// [`MLME`] group.
    ///
    /// [`MLME`]: PayloadGroupId::Mlme
    pub fn nested_information_elements(&self) -> NestedInformationElementsIterator {
        assert!(self.group_id() == PayloadGroupId::Mlme);
        NestedInformationElementsIterator::new(self.content())
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> PayloadInformationElement<T> {
    /// Clear the content of this Header Information Element.
    pub fn clear(&mut self) {
        self.data.as_mut().fill(0);
    }

    /// Set the length field value.
    pub fn set_length(&mut self, len: u16) {
        const MASK: u16 = 0b0000_0111_1111_1111;
        let b = &mut self.data.as_mut()[0..2];
        let value = u16::from_le_bytes([b[0], b[1]]) & !MASK;
        let value = value | (len & MASK);
        b.copy_from_slice(&value.to_le_bytes());
    }

    /// Set the [`PayloadGroupId`].
    pub fn set_group_id(&mut self, id: PayloadGroupId) {
        const MASK: u16 = 0b0111_1000_0000_0000;
        let b = &mut self.data.as_mut()[0..2];
        let value = u16::from_le_bytes([b[0], b[1]]) & !MASK;
        let value = value | ((id as u16) << 11) | 0b1000_0000_0000_0000;
        b.copy_from_slice(&value.to_le_bytes());
    }

    /// Return the content of this Header Information Element.
    pub fn content_mut(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[2..]
    }
}

/// Payload Information Element ID.
#[derive(Debug, Eq, PartialEq)]
pub enum PayloadGroupId {
    /// Encapsulated Service Data Unit Information Elements
    Esdu = 0x00,
    /// MAC sublayer Management Entity Information Elements
    Mlme = 0x1,
    /// Vendor specific Nested Information Elements
    VendorSpecific = 0x02,
    /// Payload Termination
    PayloadTermination = 0x0f,
    /// Unknown
    Unknown,
}

impl From<u8> for PayloadGroupId {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Esdu,
            0x01 => Self::Mlme,
            0x02 => Self::VendorSpecific,
            0x0f => Self::PayloadTermination,
            _ => Self::Unknown,
        }
    }
}

/// An [`Iterator`] over [`PayloadInformationElement`].
#[derive(Debug)]
pub struct PayloadInformationElementsIterator<'f> {
    pub(crate) data: &'f [u8],
    pub(crate) offset: usize,
    pub(crate) terminated: bool,
}

impl PayloadInformationElementsIterator<'_> {
    /// Return the offset of the iterator.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<'f> Iterator for PayloadInformationElementsIterator<'f> {
    type Item = PayloadInformationElement<&'f [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.terminated {
            None
        } else {
            let Ok(ie) = PayloadInformationElement::new(&self.data[self.offset..]) else {
                self.terminated = true;
                return None;
            };

            self.terminated = matches!(ie.group_id(), PayloadGroupId::PayloadTermination);

            self.offset += ie.length() + 2;

            if self.offset >= self.data.len() {
                self.terminated = true;
            }

            Some(ie)
        }
    }
}
