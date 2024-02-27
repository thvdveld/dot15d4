mod headers;
pub use headers::*;

mod nested;
pub use nested::*;

mod payloads;
pub use payloads::*;

use super::super::InformationElements;

use heapless::Vec;

/// A high-level representation of Information Elements.
#[derive(Debug)]
pub struct InformationElementsRepr {
    /// The header information elements.
    pub header_information_elements: Vec<HeaderInformationElementRepr, 16>,
    /// The payload information elements.
    pub payload_information_elements: Vec<PayloadInformationElementRepr, 16>,
}

impl InformationElementsRepr {
    /// Parse Information Elements.
    pub fn parse(ie: InformationElements<&[u8]>) -> Self {
        let mut header_information_elements = Vec::new();
        let mut payload_information_elements = Vec::new();

        for header_ie in ie.header_information_elements() {
            header_information_elements.push(HeaderInformationElementRepr::parse(header_ie));
        }

        for payload_ie in ie.payload_information_elements() {
            payload_information_elements.push(PayloadInformationElementRepr::parse(payload_ie));
        }

        Self {
            header_information_elements,
            payload_information_elements,
        }
    }

    /// The buffer length required to emit the Information Elements.
    pub fn buffer_len(&self) -> usize {
        let mut len = 0;

        for ie in self.header_information_elements.iter() {
            len += ie.buffer_len();
        }

        if !self.payload_information_elements.is_empty() {
            len += HeaderInformationElementRepr::HeaderTermination1.buffer_len();
        }

        for ie in self.payload_information_elements.iter() {
            len += ie.buffer_len();
        }

        len
    }

    /// Emit the Information Elements into a buffer.
    pub fn emit(&self, buffer: &mut [u8]) {
        let mut offset = 0;

        for ie in self.header_information_elements.iter() {
            ie.emit(&mut buffer[offset..][..ie.buffer_len()]);
            offset += ie.buffer_len();
        }

        if !self.payload_information_elements.is_empty() {
            HeaderInformationElementRepr::HeaderTermination1.emit(&mut buffer[offset..][..2]);
            offset += 2;
        }

        for ie in self.payload_information_elements.iter() {
            ie.emit(&mut buffer[offset..][..ie.buffer_len()]);
            offset += ie.buffer_len();
        }
    }
}
