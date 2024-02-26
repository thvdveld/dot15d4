mod headers;
pub use headers::*;

mod nested;
pub use nested::*;

mod payloads;
pub use payloads::*;

use heapless::Vec;

/// A high-level representation of Information Elements.
#[derive(Debug)]
pub struct InformationElementsRepr {
    /// The header information elements.
    pub header_information_elements: Vec<HeaderInformationElementRepr, 16>,
    /// The payload information elements.
    pub payload_information_elements: Vec<PayloadInformationElementRepr, 16>,
}
