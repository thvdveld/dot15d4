use afl::*;
use dot15d4::frame::{Frame, FrameRepr};

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(frame) = Frame::new(data) {
            let _ = FrameRepr::parse(&frame);
        }
    });
}
