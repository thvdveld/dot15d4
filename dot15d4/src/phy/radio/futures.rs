use core::cell::RefCell;
use core::mem::MaybeUninit;

use super::Radio;
use crate::phy::config::{RxConfig, TxConfig};

/// Helper structure to have cleanup logic when dropping a future
struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>, // needed such that we can have an FnOnce in drop
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: MaybeUninit::new(f),
        }
    }

    /// Consume the OnDrop such that the drop callback is not called
    pub fn defuse(mut self) {
        // Safety: Drop the function as it might have resources that need
        // cleaning. As we have sole ownership over the function, it can only
        // have been created through valid memory.
        unsafe { self.f.assume_init_drop() };
        // This prevents our own drop from being called, and may be an optimization
        core::mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        // Safety: Drop is only called once and we can only construct OnDrop from initialized memory
        unsafe { self.f.as_ptr().read()() };
    }
}

/// Convenience Future around transmitting through the radio. This future first prepares the radio,
/// then transmits before succeeding. This future, upon canceling, stops the radio from transmitting
/// and puts the radio in an IDLE state.
#[allow(clippy::await_holding_refcell_ref)]
pub async fn transmit<'task, T: AsMut<[u8]>, R: Radio>(
    radio: &'task mut R,
    data: &'task mut T,
    config: TxConfig,
) -> bool {
    let radio = RefCell::new(radio);
    // Should just work as a drop is handled at the end, after the other radio uses
    let on_drop = OnDrop::new(|| radio.borrow_mut().cancel_current_opperation());

    let mut radio = radio.borrow_mut();
    unsafe {
        radio.prepare_transmit(&config, data.as_mut()).await;
    }
    let result = radio.transmit().await;

    on_drop.defuse(); // Prevent the cancel operation from happening
    result
}

/// Convenience Future around receiving through the radio. This future first prepares the radio,
/// then receives before succeeding. This future, upon canceling, stops the radio from receiving
/// and puts the radio in an IDLE state.
#[allow(clippy::await_holding_refcell_ref)]
pub async fn receive<'task, R: Radio>(
    radio: &'task mut R,
    data: &'task mut [u8; 128],
    config: RxConfig,
) -> bool {
    let radio = RefCell::new(radio);
    // Should just work as a drop is handled at the end, after the other radio uses
    let on_drop = OnDrop::new(|| radio.borrow_mut().cancel_current_opperation());

    let mut radio = radio.borrow_mut();
    unsafe {
        radio.prepare_receive(&config, data).await;
    }
    let result = radio.receive().await;

    on_drop.defuse(); // Prevent the cancel operation from happening
    result
}
