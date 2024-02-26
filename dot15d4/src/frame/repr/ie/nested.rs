use super::super::super::{
    ChannelHopping, NestedInformationElement, NestedSubId, NestedSubIdLong, NestedSubIdShort,
    TschSlotframeAndLink, TschSynchronization, TschTimeslot,
};

/// A high-level representation of a MLME Payload Information Element.
#[derive(Debug)]
pub enum NestedInformationElementRepr {
    TschSynchronization(TschSynchronizationRepr),
    TschTimeslot(TschTimeslotRepr),
    TschSlotframeAndLink(TschSlotframeAndLinkRepr),
    ChannelHopping(ChannelHoppingRepr),
}

impl NestedInformationElementRepr {
    pub fn parse(ie: NestedInformationElement<&[u8]>) -> Self {
        match ie.sub_id() {
            NestedSubId::Short(NestedSubIdShort::TschSynchronization) => {
                Self::TschSynchronization(TschSynchronizationRepr {
                    absolute_slot_number: TschSynchronization::new(ie.content())
                        .absolute_slot_number(),
                    join_metric: TschSynchronization::new(ie.content()).join_metric(),
                })
            }
            NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
                Self::TschTimeslot(TschTimeslotRepr {
                    id: TschTimeslot::new(ie.content()).id(),
                })
            }
            NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink) => {
                Self::TschSlotframeAndLink(TschSlotframeAndLinkRepr {
                    number_of_slot_frames: TschSlotframeAndLink::new(ie.content())
                        .number_of_slot_frames(),
                })
            }
            NestedSubId::Long(NestedSubIdLong::ChannelHopping) => {
                Self::ChannelHopping(ChannelHoppingRepr {
                    hopping_sequence_id: ChannelHopping::new(ie.content()).hopping_sequence_id(),
                })
            }
            _ => todo!(),
        }
    }
}

/// A high-level representation of a TSCH Synchronization Nested Information Element.
#[derive(Debug)]
pub struct TschSynchronizationRepr {
    /// The absolute slot number (ASN).
    pub absolute_slot_number: u64,
    /// The join metric.
    pub join_metric: u8,
}

/// A high-level representation of a TSCH Timeslot Nested Information Element.
#[derive(Debug)]
pub struct TschTimeslotRepr {
    /// The timeslot ID.
    pub id: u8,
}

/// A high-level representation of a TSCH Slotframe and Link Nested Information Element.
#[derive(Debug)]
pub struct TschSlotframeAndLinkRepr {
    /// The number of slotframes.
    pub number_of_slot_frames: u8,
}

/// A high-level representation of a Channel Hopping Nested Information Element.
#[derive(Debug)]
pub struct ChannelHoppingRepr {
    /// The hopping sequence ID.
    pub hopping_sequence_id: u8,
}
