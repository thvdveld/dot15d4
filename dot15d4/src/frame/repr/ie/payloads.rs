use super::super::super::{PayloadGroupId, PayloadInformationElement};

use super::NestedInformationElementRepr;

use heapless::Vec;

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
