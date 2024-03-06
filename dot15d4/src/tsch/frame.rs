use crate::frame::*;
use crate::time::Instant;

use super::{error, trace, warn};

use heapless::Vec;

#[derive(Debug, Clone)]
pub(super) struct TschFrame {
    pub(super) id: usize,
    pub(super) data: heapless::Vec<u8, 127>,
    pub(super) timestamp: Instant,
    pub(super) slot_handle: u8,
    pub(super) frame_handle: u8,
}

/// A parsed Enhanced Beacon frame.
pub(super) struct EnhancedBeacon {
    pub(super) asn: u64,
    pub(super) join_metric: u8,
    pub(super) hopping_sequence_id: u8,
}

impl EnhancedBeacon {
    pub fn parse(frame: &Frame<&'_ [u8]>) -> Option<Self> {
        let mut asn = None;
        let mut join_metric = None;
        let mut hopping_sequence_id = None;

        let ie = frame.information_elements().unwrap();

        for payload_ie in ie.payload_information_elements() {
            if let PayloadGroupId::Mlme = payload_ie.group_id() {
                for nested_ie in payload_ie.nested_information_elements() {
                    match nested_ie.sub_id() {
                        NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink) => {
                            let slotframe_and_link =
                                TschSlotframeAndLink::new(nested_ie.content()).ok()?;
                            for sf in slotframe_and_link.slotframe_descriptors() {
                                trace!(
                                    "TSCH slotframe: handle={}, slots={}",
                                    sf.handle(),
                                    sf.link_descriptors().count(),
                                );
                            }
                        }
                        NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
                            let timeslot = TschTimeslot::new(nested_ie.content()).ok()?;
                            let id = timeslot.id();
                            if id != TschTimeslot::<&[u8]>::DEFAULT_ID {
                                // TODO: update our time slot timings with the new ones.
                                error!("TSCH non-default time slot timings not yet implemented");
                            }
                        }
                        NestedSubId::Short(NestedSubIdShort::TschSynchronization) => {
                            let synchronization =
                                TschSynchronization::new(nested_ie.content()).ok()?;
                            asn = Some(synchronization.absolute_slot_number());
                            join_metric = Some(synchronization.join_metric());
                            trace!(
                                "TSCH synchronization: asn={}, join_metric={}",
                                synchronization.absolute_slot_number(),
                                synchronization.join_metric(),
                            );
                        }
                        NestedSubId::Long(NestedSubIdLong::ChannelHopping) => {
                            let channel_hopping = ChannelHopping::new(nested_ie.content()).ok()?;
                            hopping_sequence_id = Some(channel_hopping.hopping_sequence_id());
                            trace!(
                                "TSCH channel hopping: hopping_sequence_id={}",
                                channel_hopping.hopping_sequence_id(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        if let (Some(asn), Some(join_metric), Some(hopping_sequence_id)) =
            (asn, join_metric, hopping_sequence_id)
        {
            Some(Self {
                asn,
                join_metric,
                hopping_sequence_id,
            })
        } else {
            None
        }
    }
}
