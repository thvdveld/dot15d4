use dot15d4_frame::{Address, DataFrame, FrameType, FrameVersion};

use crate::sync::channel::Sender;
use crate::sync::mutex::{Mutex, MutexGuard};

use crate::mac::constants::*;

/// Acquire the lock on Mutex by repeatably calling the wants_lock channel to
/// request access. The resulting guard will be made available through the
/// out_guard and will first check if the guard is not already in our
/// possession. If it is, spinning will be performed.
pub async fn acquire_lock<'a, 'b, T>(
    mutex: &'a Mutex<T>,
    wants_lock: &Sender<'b, ()>,
    out_guard: &mut Option<MutexGuard<'a, T>>,
) {
    match out_guard {
        Some(_) => (),
        None => {
            'inner: loop {
                // repeatably ask for the lock, as this might need a few tries to prevent
                // deadlocks
                match mutex.try_lock() {
                    Some(guard) => {
                        // wants_to_transmit_signal.reset();

                        // reset signal, such that the receiving end may continue the next time it
                        // acquires the lock
                        *out_guard = Some(guard);
                        return;
                    }
                    None => {
                        // Ask the receiving loop to let go of the radio
                        wants_lock.send_async(()).await;
                        // yield_now().await; // Give the receiving end time to react
                        continue 'inner;
                    }
                }
            }
        }
    }
}

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
