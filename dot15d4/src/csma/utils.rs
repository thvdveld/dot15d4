use crate::sync::channel::Sender;
use crate::sync::mutex::{Mutex, MutexGuard};

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
