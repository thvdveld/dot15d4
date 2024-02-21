use super::TschFrame;

use super::warn;

pub(super) struct FrameBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    next_id: usize,
}

impl<T, const N: usize> Default for FrameBuffer<T, N> {
    fn default() -> Self {
        Self {
            buffer: core::array::from_fn(|_| None),
            next_id: 0,
        }
    }
}

impl<T, const N: usize> FrameBuffer<T, N> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> FrameBuffer<TschFrame, N> {
    pub fn add_frame(&mut self, mut frame: TschFrame) {
        for i in 0..N {
            if self.buffer[i].is_none() {
                frame.id = self.next_id;
                self.buffer[i] = Some(frame);
                self.next_id = self.next_id.wrapping_add(1);
                return;
            }
        }

        warn!("Frame buffer full");
    }

    /// Get a frame for a given slot and frame handle, based on its timestamp.
    pub fn get_frame_for_slot(&self, frame_handle: u8, slot_handle: u8) -> Option<&TschFrame> {
        self.buffer
            .iter()
            .flatten()
            .filter(|f| f.frame_handle == frame_handle && f.slot_handle == slot_handle)
            .min_by_key(|f| f.timestamp)
    }

    /// Remove a frame from the buffer based on its ID.
    pub fn remove_frame(&mut self, id: usize) -> Option<TschFrame> {
        for i in 0..N {
            if let Some(frame) = self.buffer[i].take() {
                if frame.id == id {
                    return Some(frame);
                } else {
                    self.buffer[i] = Some(frame);
                }
            }
        }

        None
    }
}
