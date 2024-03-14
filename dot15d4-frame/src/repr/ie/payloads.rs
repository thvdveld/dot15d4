use super::super::super::{Error, Result};
use super::super::super::{NestedInformationElement, PayloadGroupId, PayloadInformationElement};

use super::NestedInformationElementRepr;

use heapless::Vec;

/// A high-level representation of a Payload Information Element.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum PayloadInformationElementRepr {
    Mlme(Vec<NestedInformationElementRepr, 16>),
    PayloadTermination,
}

#[cfg(feature = "fuzz")]
impl arbitrary::Arbitrary<'_> for PayloadInformationElementRepr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        match u.int_in_range(0..=1)? {
            0 => Ok(Self::PayloadTermination),
            _ => {
                let mut nested_information_elements = Vec::new();

                for _ in 0..u.int_in_range(0..=15)? {
                    nested_information_elements
                        .push(NestedInformationElementRepr::arbitrary(u)?)
                        .map_err(|_err| arbitrary::Error::IncorrectFormat)?;
                }

                Ok(Self::Mlme(nested_information_elements))
            }
        }
    }
}

impl PayloadInformationElementRepr {
    /// Parse a Payload Information Element.
    pub fn parse(ie: &PayloadInformationElement<&[u8]>) -> Result<Self> {
        match ie.group_id() {
            PayloadGroupId::Mlme => {
                let mut nested_information_elements = Vec::new();

                for nested_ie in ie.nested_information_elements() {
                    if nested_information_elements
                        .push(NestedInformationElementRepr::parse(&nested_ie)?)
                        .is_err()
                    {
                        break;
                    }
                }

                Ok(Self::Mlme(nested_information_elements))
            }
            _ => Err(Error),
        }
    }

    /// The buffer length required to emit the Payload Information Element.
    pub fn buffer_len(&self) -> usize {
        2 + self.inner_len()
    }

    /// The buffer length required to emit the inner part of the Payload
    /// Information Element.
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
    pub fn emit(&self, w: &mut PayloadInformationElement<&mut [u8]>) {
        w.clear();
        w.set_length(self.inner_len() as u16);
        w.set_group_id(self.into());

        let buffer = w.content_mut();
        match self {
            Self::Mlme(nested_ies) => {
                let mut offset = 0;
                for ie in nested_ies.iter() {
                    ie.emit(&mut NestedInformationElement::new_unchecked(
                        &mut buffer[offset..],
                    ));
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
