use crate::phy::FrameBuffer;

use super::mcps::data::DataIndication;
pub use super::mcps::data::DataRequest;
use super::mlme::beacon::{BeaconNotifyIndication, BeaconRequest};
use super::mlme::set::SetRequestAttribute;

/// Enum representing all (currently) supported MAC commands
pub enum MacRequest {
    McpsDataRequest(DataRequest),
    MlmeBeaconRequest(BeaconRequest),
    MlmeSetRequest(SetRequestAttribute),
    EmptyRequest,
}

impl Default for MacRequest {
    fn default() -> Self {
        Self::McpsDataRequest(DataRequest {
            buffer: FrameBuffer::default(),
        })
    }
}

pub enum MacIndication {
    McpsData(DataIndication),
    MlmeBeaconNotify(BeaconNotifyIndication),
}
