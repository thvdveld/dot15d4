pub mod constants;
pub mod csma;
pub mod utils;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// Cca failed, resulting in a backoff (nth try)
    CcaBackoff(u16),
    /// Cca failed after to many fallbacks
    CcaFailed,
    /// Ack failed, resulting in a retry later (nth try)
    AckRetry(u16),
    /// Ack failed, after to many retransmissions
    AckFailed,
    /// The buffer did not follow the correct device structure
    InvalidDeviceStructure,
    /// Invalid IEEE frame
    InvalidIEEEStructure,
    /// Something went wrong
    Error,
}
