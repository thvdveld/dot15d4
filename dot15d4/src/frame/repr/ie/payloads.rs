use super::super::super::{NestedInformationElement, PayloadGroupId, PayloadInformationElement};

use super::NestedInformationElementRepr;

use heapless::Vec;

/// A high-level representation of a Payload Information Element.
#[derive(Debug)]
pub enum PayloadInformationElementRepr {
    Mlme(Vec<NestedInformationElementRepr, 16>),
    PayloadTermination,
}

impl PayloadInformationElementRepr {
    /// Parse a Payload Information Element.
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

    /// The buffer length required to emit the Payload Information Element.
    pub fn buffer_len(&self) -> usize {
        2 + self.inner_len()
    }

    fn inner_len(&self) -> usize {
        match self {
            Self::Mlme(nested_ies) => {
                let mut len = 0;

                for ie in nested_ies.iter() {
                    len += ie.buffer_len();
                }

                len
            }
            Self::PayloadTermination => 0,
        }
    }

    /// Emit the Payload Information Element into a buffer.
    pub fn emit(&self, buffer: &mut [u8]) {
        let mut w = PayloadInformationElement::new_unchecked(buffer);
        w.clear();
        w.set_length(self.inner_len() as u16);
        w.set_group_id(self.into());

        let buffer = w.content_mut();
        match self {
            Self::Mlme(nested_ies) => {
                let mut offset = 0;
                for ie in nested_ies.iter() {
                    ie.emit(&mut buffer[offset..]);
                    offset += ie.buffer_len();
                }
            }
            Self::PayloadTermination => todo!(),
        }
    }
}

impl From<&PayloadInformationElementRepr> for PayloadGroupId {
    fn from(val: &PayloadInformationElementRepr) -> Self {
        use PayloadInformationElementRepr::*;
        match val {
            Mlme(_) => PayloadGroupId::Mlme,
            PayloadTermination => PayloadGroupId::PayloadTermination,
        }
    }
}
