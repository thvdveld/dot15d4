#![no_std]

use core::{future::poll_fn, task::Poll};

/// Simple function that makes the current task yield immediatly, such that
/// other tasks can have the opportunity to make progress
pub async fn yield_now() {
    let mut has_yielded = false;
    poll_fn(move |cx| {
        if has_yielded {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref(); // Make sure we get called again soon
            has_yielded = true;
            Poll::Pending
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use pollster::FutureExt;

    #[test]
    pub fn test_yield_finishes() {
        assert!(
            async {
                yield_now();
                true
            }
            .block_on(),
            "Yield should fininsh"
        );
    }
}
