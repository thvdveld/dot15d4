use dot15d4_frame::{Address, DataFrame, FrameType, FrameVersion};

use crate::mac::constants::*;

/// Checks if the current frame is intended for us. For the hardware
/// address, the full 64-bit address should be provided.
pub fn is_frame_for_us(hardware_address: &[u8; 8], frame: &DataFrame<&'_ [u8]>) -> bool {
    // Check if the type is known, otherwise drop
    if matches!(frame.frame_control().frame_type(), FrameType::Unknown) {
        return false;
    }
    // Check if the Frame version is valid, otherwise drop
    if matches!(frame.frame_control().frame_version(), FrameVersion::Unknown) {
        return false;
    }

    let addr = match frame.addressing().and_then(|fields| fields.dst_address()) {
        Some(addr) => addr,
        None if MAC_IMPLICIT_BROADCAST => Address::BROADCAST,
        _ => return false,
    };

    // Check if dst_pan (in present) is provided
    let dst_pan_id = frame
        .addressing()
        .and_then(|fields| fields.dst_pan_id())
        .unwrap_or(BROADCAST_PAN_ID);
    if dst_pan_id != MAC_PAN_ID && dst_pan_id != BROADCAST_PAN_ID {
        return false;
    }

    // TODO: Check rules if frame comes from PAN coordinator and the same MAC_PAN_ID
    // TODO: Implement `macGroupRxMode` check here
    match &addr {
        _ if addr.is_broadcast() => true,
        Address::Absent => false,
        Address::Short(addr) => hardware_address[6..] == addr[..2],
        Address::Extended(addr) => hardware_address == addr,
    }
}
