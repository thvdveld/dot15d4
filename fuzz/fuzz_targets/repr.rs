#![no_main]

use dot15d4_frame::{DataFrame, FrameRepr};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|repr: FrameRepr| {
    if repr.validate().is_err() {
        return;
    }

    let len = repr.buffer_len();
    let mut buffer = vec![0; len];
    repr.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));
});
