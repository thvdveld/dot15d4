use core::ops::RangeInclusive;

use embedded_hal_async::delay::DelayNs;
use rand_core::RngCore;

use crate::mac::MacService;
use crate::phy::radio::{Radio, RadioFrameMut};
use crate::upper::UpperLayer;

pub enum ScanType {
    Ed,
    Active,
    Passive,
    Orphan,
    EnhancedActiveScan,
}

pub enum ScanChannels {
    All,
    Single(u8),
}

#[allow(dead_code)]
pub struct ScanConfirm {
    scan_type: ScanType,
    channel_page: u8,
}
pub enum ScanError {
    // TODO: not supported
    LimitReached,
    // TODO: not supported
    NoBeacon,
    // TODO: not supported
    ScanInProgress,
    // TODO: not supported
    CounterError,
    // TODO: not supported
    FrameTooLong,
    // TODO: not supported
    BadChannel,
    // TODO: not supported
    InvalidParameter,
}

#[allow(dead_code)]
impl<Rng, U, TIMER, R> MacService<'_, Rng, U, TIMER, R>
where
    Rng: RngCore,
    U: UpperLayer,
    TIMER: DelayNs + Clone,
    R: Radio,
    for<'a> R::RadioFrame<&'a mut [u8]>: RadioFrameMut<&'a mut [u8]>,
    for<'a> R::TxToken<'a>: From<&'a mut [u8]>,
{
    /// Initiates a channel scan over a given set of channels.
    pub(crate) async fn mlme_scan_request(
        &self,
        _scan_type: ScanType,
        _scan_channels: ScanChannels,
        _scan_duration: u8,
        _channel_page: u8,
    ) -> Result<ScanConfirm, ScanError> {
        Err(ScanError::InvalidParameter)
    }
}

// Implement IntoIterator for Channels so you can write: for x in scan_channels { ... }
impl IntoIterator for ScanChannels {
    type Item = u8;
    type IntoIter = RangeInclusive<u8>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ScanChannels::All => 11_u8..=26,
            ScanChannels::Single(ch) => ch..=ch,
        }
    }
}
