use super::{NestedInformationElementRepr, NestedInformationElementsIterator};
use heapless::Vec;

/// A reader/writer for the IEEE 802.15.4 Payload Information Elements.
#[derive(Debug, Eq, PartialEq)]
pub struct PayloadInformationElement<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> PayloadInformationElement<T> {
    /// Return the length field value.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
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
        &self.data.as_ref()[2..][..self.len()]
    }

    /// Returns [`NestedInformationElementsIterator`] [`Iterator`].
    ///
    /// ## Panics
    /// This method panics if the [`PayloadInformationElement`] is not an [`MLME`] group.
    ///
    /// [`MLME`]: PayloadGroupId::Mlme
    pub fn nested_information_elements(&self) -> NestedInformationElementsIterator {
        assert!(self.group_id() == PayloadGroupId::Mlme);
        NestedInformationElementsIterator::new(self.content())
    }
}

impl<T: AsRef<[u8]>> core::fmt::Display for PayloadInformationElement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.group_id() {
            PayloadGroupId::Mlme => {
                writeln!(f, "{:?}", self.group_id())?;

                for nested in self.nested_information_elements() {
                    writeln!(f, "  {}", nested)?;
                }

                Ok(())
            }
            id => write!(f, "{:?}({:0x?})", id, self.content()),
        }
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
            let ie = PayloadInformationElement {
                data: &self.data[self.offset..],
            };

            self.terminated = matches!(ie.group_id(), PayloadGroupId::PayloadTermination);

            self.offset += ie.len() + 2;

            if self.offset >= self.data.len() {
                self.terminated = true;
            }

            Some(ie)
        }
    }
}

/// A high-level representation of a Payload Information Element.
#[derive(Debug)]
pub enum PayloadInformationElementRepr {
    Mlme(Vec<NestedInformationElementRepr, 16>),
}

impl PayloadInformationElementRepr {
    pub fn parse(ie: PayloadInformationElement<&[u8]>) -> Self {
        match ie.group_id() {
            PayloadGroupId::Mlme => {
                let mut nested_information_elements = Vec::new();

                for nested_ie in ie.nested_information_elements() {
                    nested_information_elements
                        .push(NestedInformationElementRepr::parse(nested_ie));
                }

                Self::Mlme(nested_information_elements)
            }
            _ => todo!(),
        }
    }
}
