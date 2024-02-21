//! Information Elements readers and writers.

mod headers;
pub use headers::*;

mod payloads;
pub use payloads::*;

mod nested;
pub use nested::*;

use heapless::Vec;

/// IEEE 802.15.4 Information Element reader.
pub struct InformationElements<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> InformationElements<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        let mut len = 0;

        let mut iter = self.header_information_elements();
        while iter.next().is_some() {}
        len += iter.offset();

        let mut iter = self.payload_information_elements();
        while iter.next().is_some() {}
        len += iter.offset();

        len
    }

    /// Returns an [`Iterator`] over [`HeaderInformationElement`].
    pub fn header_information_elements(&self) -> HeaderInformationElementsIterator {
        HeaderInformationElementsIterator {
            data: self.data.as_ref(),
            offset: 0,
            terminated: self.data.as_ref().is_empty(),
        }
    }

    /// Returns an [`Iterator`] over [`PayloadInformationElement`].
    pub fn payload_information_elements(&self) -> PayloadInformationElementsIterator {
        let start = self
            .header_information_elements()
            .map(|ie| ie.len() + 2)
            .sum::<usize>();

        let terminated = start >= self.data.as_ref().len();

        PayloadInformationElementsIterator {
            data: &self.data.as_ref()[start..],
            offset: 0,
            terminated,
        }
    }
}

/// A high-level representation of Information Elements.
#[derive(Debug)]
pub struct InformationElementsRepr {
    /// The header information elements.
    pub header_information_elements: Vec<HeaderInformationElementRepr, 16>,
    /// The payload information elements.
    pub payload_information_elements: Vec<PayloadInformationElementRepr, 16>,
}
