use super::super::super::{
    ChannelHopping, NestedInformationElement, NestedSubId, NestedSubIdLong, NestedSubIdShort,
    TschSlotframeAndLink, TschSynchronization, TschTimeslot,
};
use super::super::super::{Error, Result};

/// A high-level representation of a MLME Payload Information Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum NestedInformationElementRepr {
    /// TSCH Synchronization Information Element.
    TschSynchronization(TschSynchronizationRepr),
    /// TSCH Timeslot Information Element.
    TschTimeslot(TschTimeslotRepr),
    /// TSCH Slotframe and Link Information Element.
    TschSlotframeAndLink(TschSlotframeAndLinkRepr),
    /// Channel Hopping Information Element.
    ChannelHopping(ChannelHoppingRepr),
}

impl NestedInformationElementRepr {
    /// Parse a Nested Information Element.
    pub fn parse(ie: &NestedInformationElement<&[u8]>) -> Result<Self> {
        Ok(match ie.sub_id() {
            NestedSubId::Short(NestedSubIdShort::TschSynchronization) => Self::TschSynchronization(
                TschSynchronizationRepr::parse(&TschSynchronization::new(ie.content())?),
            ),
            NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
                Self::TschTimeslot(TschTimeslotRepr::parse(&TschTimeslot::new(ie.content())?))
            }
            NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink) => {
                Self::TschSlotframeAndLink(TschSlotframeAndLinkRepr::parse(
                    &TschSlotframeAndLink::new(ie.content())?,
                ))
            }
            NestedSubId::Long(NestedSubIdLong::ChannelHopping) => Self::ChannelHopping(
                ChannelHoppingRepr::parse(&ChannelHopping::new(ie.content())?),
            ),
            _id => {
                #[cfg(feature = "panic")]
                {
                    panic!("unsupported Nested Information Element: {_id:?}");
                }
                #[allow(unreachable_code)]
                return Err(Error);
            }
        })
    }

    /// The buffer length required to emit the Nested Information Element.
    pub fn buffer_len(&self) -> usize {
        2 + self.inner_len()
    }

    /// The buffer length required to emit the inner part of the Nested
    /// Information Element.
    pub fn inner_len(&self) -> usize {
        match self {
            Self::TschSynchronization(repr) => repr.buffer_len(),
            Self::TschTimeslot(repr) => repr.buffer_len(),
            Self::TschSlotframeAndLink(repr) => repr.buffer_len(),
            Self::ChannelHopping(repr) => repr.buffer_len(),
        }
    }

    /// Emit the Nested Information Element into a buffer.
    pub fn emit(&self, w: &mut NestedInformationElement<&mut [u8]>) {
        let id = NestedSubId::from(self);

        w.clear();
        w.set_length(self.inner_len() as u16, id);
        w.set_sub_id(id);

        match self {
            Self::TschSynchronization(repr) => {
                repr.emit(&mut TschSynchronization::new_unchecked(w.content_mut()))
            }
            Self::TschTimeslot(repr) => {
                repr.emit(&mut TschTimeslot::new_unchecked(w.content_mut()))
            }
            Self::TschSlotframeAndLink(repr) => {
                repr.emit(&mut TschSlotframeAndLink::new_unchecked(w.content_mut()))
            }
            Self::ChannelHopping(repr) => {
                repr.emit(&mut ChannelHopping::new_unchecked(w.content_mut()))
            }
        }
    }
}

impl From<&NestedInformationElementRepr> for NestedSubId {
    fn from(value: &NestedInformationElementRepr) -> Self {
        match value {
            NestedInformationElementRepr::TschSynchronization(_) => {
                NestedSubId::Short(NestedSubIdShort::TschSynchronization)
            }
            NestedInformationElementRepr::TschTimeslot(_) => {
                NestedSubId::Short(NestedSubIdShort::TschTimeslot)
            }
            NestedInformationElementRepr::TschSlotframeAndLink(_) => {
                NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink)
            }
            NestedInformationElementRepr::ChannelHopping(_) => {
                NestedSubId::Long(NestedSubIdLong::ChannelHopping)
            }
        }
    }
}

/// A high-level representation of a TSCH Synchronization Nested Information
/// Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TschSynchronizationRepr {
    /// The absolute slot number (ASN).
    pub absolute_slot_number: u64,
    /// The join metric.
    pub join_metric: u8,
}

impl TschSynchronizationRepr {
    /// Parse a TSCH Synchronization Information Element.
    pub fn parse(ie: &TschSynchronization<&[u8]>) -> Self {
        Self {
            absolute_slot_number: ie.absolute_slot_number(),
            join_metric: ie.join_metric(),
        }
    }

    /// The buffer length required to emit the TSCH Synchronization Information
    /// Element.
    pub const fn buffer_len(&self) -> usize {
        6
    }

    /// Emit the TSCH Synchronization Information Element into a buffer.
    pub fn emit(&self, ie: &mut TschSynchronization<&mut [u8]>) {
        ie.set_absolute_slot_number(self.absolute_slot_number);
        ie.set_join_metric(self.join_metric);
    }
}

/// A high-level representation of a TSCH Slotframe and Link Nested Information
/// Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct TschSlotframeAndLinkRepr {
    /// The number of slotframes.
    pub number_of_slotframes: u8,
}

impl TschSlotframeAndLinkRepr {
    /// Parse a TSCH Slotframe and Link Information Element.
    pub fn parse(ie: &TschSlotframeAndLink<&[u8]>) -> Self {
        Self {
            number_of_slotframes: ie.number_of_slotframes(),
        }
    }

    /// The buffer length required to emit the TSCH Slotframe and Link
    /// Information Element.
    pub fn buffer_len(&self) -> usize {
        1
    }

    /// Emit the TSCH Slotframe and Link Information Element into a buffer.
    pub fn emit(&self, ie: &mut TschSlotframeAndLink<&mut [u8]>) {
        ie.set_number_of_slotframes(self.number_of_slotframes);
    }
}

/// A high-level representation of a TSCH Timeslot Nested Information Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
/// A high-level representation of a TSCH Timeslot Nested Information Element.
#[derive(Debug)]
pub enum TschTimeslotRepr {
    /// Default Timeslot Template with given ID. ID shall be 0.
    Default(u8),
    /// Custom Timeslot timings
    Custom(TschTimeslotTimings),
}

impl TschTimeslotRepr {
    /// Parse a TSCH Timeslot Information Element.
    pub fn parse(ie: &TschTimeslot<&[u8]>) -> Self {
        if ie.has_timeslot_timings() {
            Self::Custom(ie.timeslot_timings())
        } else {
            Self::Default(ie.id())
        }
    }

    /// The buffer length required to emit the TSCH Timeslot Information
    /// Element.
    pub fn buffer_len(&self) -> usize {
        match self {
            Self::Default(_id) => 1,
            Self::Custom(timings) => {
                let max_tx = timings.max_tx().as_us() as u32;
                let timeslot_length = timings.timeslot_length().as_us() as u32;
                if max_tx > 65535 || timeslot_length > 65535 {
                    27
                } else {
                    25
                }
            }
        }
    }

    /// Emit the TSCH Timeslot Information Element into a buffer.
    pub fn emit(&self, ie: &mut TschTimeslot<&mut [u8]>) {
        match self {
            Self::Default(id) => {
                ie.set_timeslot_id(*id);
            }
            Self::Custom(timings) => {
                ie.set_timeslot_timings(timings);
            }
        }
    }
}

#[cfg(feature = "fuzz")]
impl arbitrary::Arbitrary<'_> for TschTimeslotRepr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        match u.int_in_range(0..=1)? {
            0 => Ok(Self::Default(0)),
            _ => {
                let mut timings =
                    TschTimeslotTimings::new(u.int_in_range(1..=255)?, Duration::from_us(0));
                // TODO: set random values that are coherent
                let guard_time = Duration::from_us(u.int_in_range(2000..=2400)?);
                let offset = Duration::from_us(u.int_in_range(2100..=2140)?);
                timings.set_cca_offset(Duration::from_us(u.int_in_range(1750..=1850)?));
                timings.set_cca(Duration::from_us(128));
                timings.set_tx_offset(offset);
                timings.set_rx_offset(offset - (guard_time / 2));
                timings.set_rx_ack_delay(Duration::from_us(u.int_in_range(780..=820)?));
                timings.set_tx_ack_delay(Duration::from_us(u.int_in_range(980..=1020)?));
                timings.set_rx_wait(guard_time);
                timings.set_ack_wait(Duration::from_us(u.int_in_range(380..=420)?));
                timings.set_rx_tx(Duration::from_us(u.int_in_range(190..=194)?));
                timings.set_max_ack(Duration::from_us(u.int_in_range(2350..=2450)?));
                timings.set_max_tx(Duration::from_us(u.int_in_range(4200..=4300)?));
                timings
                    .set_timeslot_length(Duration::from_us(10000 + 1000 * u.int_in_range(0..=10)?));
                Ok(Self::Custom(timings))
            }
        }
    }
}

/// A high-level representation of a Channel Hopping Nested Information Element.
#[derive(Debug)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ChannelHoppingRepr {
    /// The hopping sequence ID.
    pub hopping_sequence_id: u8,
}

impl ChannelHoppingRepr {
    /// Parse a Channel Hopping Information Element.
    pub fn parse(ie: &ChannelHopping<&[u8]>) -> Self {
        Self {
            hopping_sequence_id: ie.hopping_sequence_id(),
        }
    }

    /// The buffer length required to emit the Channel Hopping Information
    /// Element.
    pub fn buffer_len(&self) -> usize {
        1
    }

    /// Emit the Channel Hopping Information Element into a buffer.
    pub fn emit(&self, ie: &mut ChannelHopping<&mut [u8]>) {
        ie.set_hopping_sequence_id(self.hopping_sequence_id);
    }
}
