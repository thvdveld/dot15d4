#![no_main]

use dot15d4_frame::{Frame, FrameRepr};

use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: &[u8]| -> Corpus {
    if data.len() > 127 {
        return Corpus::Reject;
    }

    if let Ok(frame) = Frame::new(data) {
        let _ = FrameRepr::parse(&frame);
    }

    Corpus::Keep
});
