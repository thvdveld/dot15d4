#![no_std]

use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::future::Future;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::pin;
use core::pin::Pin;
use core::task::Waker;
use core::task::{Context, Poll};

struct MutexState {
    locked: bool,
    waker: Option<Waker>,
}

/// A generic mutex that is independent on the underlying async runtime.
/// The idea is that this is used to synchronize different parts inside 1 single task that may run concurrently through `select`.
pub struct Mutex<T> {
    value: UnsafeCell<T>,
    state: RefCell<MutexState>,
    _no_send_sync: PhantomData<*mut T>, // Probably not needed as we have `UnsafeCell`
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Mutex {
            value: UnsafeCell::new(value),
            state: RefCell::new(MutexState {
                locked: false,
                waker: None,
            }),
            _no_send_sync: PhantomData,
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, T> {
        // Wait until we can acquire the lock
        LockFuture { mutex: self }.await;

        // Now that we have acquired the lock, we can return the mutex
        println!("Lock acquired (lock)");
        MutexGuard { mutex: self }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        let mut state = self.state.borrow_mut();
        if !state.locked {
            // The lock is currently not yet locked -> acquire lock
            state.locked = true;
            println!("Lock acquired (try_lock)");
            Some(MutexGuard { mutex: self })
        } else {
            // The current lock is locked, return None
            None
        }
    }

    /// Get access to the protected value inside the mutex. This is similar to
    /// the Mutex::get_mut in std.
    pub fn get_mut(&mut self) -> &mut T {
        // Safety: &mut gives us exclusive access to T
        self.value.get_mut()
    }

    /// # Safety
    /// Only use this method if you are sure there are no locks currently taken. If you have a mutable reference, prefer to use the `get_mut` method instead.
    pub unsafe fn read(&self) -> &T {
        &*self.value.get()
    }
}

/// Represesnts current exclusive access to the resource protected by a mutex
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        /// Safety: Only one mutex can exist at a time
        unsafe {
            &*self.mutex.value.get()
        }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        /// Safety: Only one mutex can exist at a time
        unsafe {
            &mut *self.mutex.value.get()
        }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        let mut mutex_state = self.mutex.state.borrow_mut();
        println!("Lock dropped");

        // Release the lock
        mutex_state.locked = false;

        // Call the waker if needed
        if let Some(waker) = mutex_state.waker.take() {
            waker.wake()
        }
    }
}

struct LockFuture<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut mutex_state = self.mutex.state.borrow_mut();
        if mutex_state.locked {
            // Mutex is locked here, wake the previous task, so it can make progress and we do not have to remember it
            let new_waker = cx.waker();
            match &mut mutex_state.waker {
                // We already have the same waker stored, do not wake
                Some(waker) if waker.will_wake(new_waker) => {
                    waker.clone_from(new_waker);
                }
                // New waker, wake the previous and store current
                waker @ Some(_) => {
                    waker.take().unwrap().wake();
                    *waker = Some(new_waker.clone());
                }
                // No waker yet, store the new one
                waker @ None => *waker = Some(new_waker.clone()),
            };

            // Mutex is locked, keep waiting
            Poll::Pending
        } else {
            // Mutex is unlocked, lock it
            mutex_state.locked = true;

            Poll::Ready(())
        }
    }
}

#[cfg(test)]
mod tests {
    use pollster::FutureExt as _;

    use crate::sync::{join::join, select::select, yield_now};

    use super::Mutex;

    #[test]
    pub fn test_mutex_no_concurrency() {
        async {
            let mut mutex = Mutex::new(0usize);
            {
                let mut guard = mutex.lock().await;
                *guard += 1;
                assert_eq!(*guard, 1, "The guard should be readable");
            }

            assert_eq!(
                *mutex.get_mut(),
                1,
                "The internal mutex should have been updated"
            )
        }
        .block_on()
    }

    #[test]
    pub fn test_mutex_select_concurrency() {
        async {
            let mut mutex = Mutex::new(0usize);
            for _ in 0..100 {
                select(
                    async {
                        let mut guard = mutex.lock().await;
                        *guard += 1;
                    },
                    async {
                        let mut guard = mutex.lock().await;
                        *guard += 1;
                    },
                )
                .await;
            }

            assert_eq!(*mutex.get_mut(), 100);
        }
        .block_on()
    }

    #[test]
    pub fn test_mutex_join_concurrency() {
        async {
            let mut mutex = Mutex::new(0usize);
            for _ in 0..100 {
                join(
                    async {
                        let mut guard = mutex.lock().await;
                        *guard += 1;
                    },
                    async {
                        let mut guard = mutex.lock().await;
                        *guard += 1;
                    },
                )
                .await;
            }

            assert_eq!(*mutex.get_mut(), 200);
        }
        .block_on()
    }

    #[test]
    pub fn test_try_lock() {
        async {
            let mut mutex = Mutex::new(0usize);
            join(
                async {
                    let mut guard = mutex.lock().await;
                    // Keep lock for 10 iterations
                    for _ in 0..10 {
                        *guard += 1;
                        yield_now::yield_now().await;
                    }
                },
                async {
                    let mut i = 0;
                    loop {
                        if let Some(mut guard) = mutex.try_lock() {
                            *guard += 1;
                            break;
                        }

                        if i == 20 {
                            panic!("Try lock takes to long!");
                        }

                        yield_now::yield_now().await;
                    }
                },
            )
            .await;

            assert_eq!(*mutex.get_mut(), 11);
        }
        .block_on()
    }

    #[test]
    /// Check with Miri whether or not drop is called correctly. If true, then all heap allocation should be deallocated correctly
    pub fn test_drop_by_leaking() {
        async {
            let mut mutex = Mutex::new(Box::new(0));
            let mut guard = mutex.lock().await;
        }
        .block_on()
    }
}
