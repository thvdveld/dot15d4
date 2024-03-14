//! Information Elements readers and writers.

mod headers;
pub use headers::*;

mod payloads;
pub use payloads::*;

mod nested;
pub use nested::*;

use super::{Error, Result};

/// IEEE 802.15.4 Information Element reader.
pub struct InformationElements<T: AsRef<[u8]>> {
    data: T,
}

impl<T: AsRef<[u8]>> InformationElements<T> {
    /// Create a new [`InformationElements`] reader from a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short to contain the information
    /// elements.
    pub fn new(data: T) -> Result<Self> {
        let ie = Self::new_unchecked(data);

        if !ie.check_len() {
            return Err(Error);
        }

        Ok(ie)
    }

    /// Returns `false` if the buffer is too short to contain the information
    /// elements.
    fn check_len(&self) -> bool {
        let mut len = 0;

        let mut iter = self.header_information_elements();
        while iter.next().is_some() {}
        len += iter.offset();

        if len > self.data.as_ref().len() {
            return false;
        }

        let mut iter = self.payload_information_elements();
        while iter.next().is_some() {}
        len += iter.offset();

        self.data.as_ref().len() >= len
    }

    /// Create a new [`InformationElements`] reader from a given buffer without
    /// length checking.
    pub fn new_unchecked(data: T) -> Self {
        Self { data }
    }

    /// Returns the length of the information elements.
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
