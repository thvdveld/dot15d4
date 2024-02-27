mod headers;
pub use headers::*;

mod nested;
pub use nested::*;

mod payloads;
pub use payloads::*;

use super::super::InformationElements;

use heapless::Vec;

/// A high-level representation of Information Elements.
#[derive(Debug, Default)]
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

    /// The header terminations required to emit the Information Elements.
    /// The first bool is the HT1, the second is the HT2, and the third is the PT.
    fn header_terminations(&self, contains_payload: bool) -> (bool, bool, bool) {
        match (
            !self.header_information_elements.is_empty(),
            !self.payload_information_elements.is_empty(),
            contains_payload,
        ) {
            // No IE lists, so no terminations.
            (false, false, false) => (false, false, false),
            // Only header IE list. The end of the frame can be determined by the length of the
            // frame.
            (true, false, false) => (false, false, false),
            // Only payload IE list. The HT1 is required to terminate the header IE list.
            (false, true, false) => (true, false, false),
            // Both IE lists. The HT1 is required to terminate the header IE list.
            // The payload HT is optional.
            (true, true, false) => (true, false, false),
            // No IE lists, so no terminations.
            (false, false, true) => (false, false, false),
            // No payload IE list. The HT2 is required to terminate the header IE list.
            (true, false, true) => (false, true, false),
            // No header IE list. The HT1 is required to terminate the payload IE list.
            // The payload HT is optional.
            (false, true, true) => (true, false, true),
            // Both IE lists. The HT1 is required to terminate the header IE list.
            // The payload HT is optional.
            (true, true, true) => (true, false, true),
        }
    }

    /// The buffer length required to emit the Information Elements.
    pub fn buffer_len(&self, contains_payload: bool) -> usize {
        let mut len = 0;

        let (ht1, ht2, pt) = self.header_terminations(contains_payload);

        for ie in self.header_information_elements.iter() {
            len += ie.buffer_len();
        }

        if ht1 {
            len += HeaderInformationElementRepr::HeaderTermination1.buffer_len();
        }

        if ht2 {
            len += HeaderInformationElementRepr::HeaderTermination1.buffer_len();
        }

        for ie in self.payload_information_elements.iter() {
            len += ie.buffer_len();
        }

        if pt {
            len += PayloadInformationElementRepr::PayloadTermination.buffer_len();
        }

        len
    }

    /// Emit the Information Elements into a buffer.
    pub fn emit(&self, buffer: &mut [u8], contains_payload: bool) {
        let mut offset = 0;

        let (ht1, ht2, pt) = self.header_terminations(contains_payload);

        for ie in self.header_information_elements.iter() {
            ie.emit(&mut buffer[offset..][..ie.buffer_len()]);
            offset += ie.buffer_len();
        }

        if ht1 {
            HeaderInformationElementRepr::HeaderTermination1.emit(&mut buffer[offset..][..2]);
            offset += 2;
        }

        if ht2 {
            HeaderInformationElementRepr::HeaderTermination2.emit(&mut buffer[offset..][..2]);
            offset += 2;
        }

        for ie in self.payload_information_elements.iter() {
            ie.emit(&mut buffer[offset..][..ie.buffer_len()]);
            offset += ie.buffer_len();
        }

        if pt {
            PayloadInformationElementRepr::PayloadTermination.emit(&mut buffer[offset..][..2]);
            offset += 2;
        }
    }
}
