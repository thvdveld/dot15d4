use afl::*;
use dot15d4::frame::{Frame, FrameRepr};

fn main() {
    fuzz!(|repr: FrameRepr| {
        println!("{:#?}", repr);
        let len = repr.buffer_len();
        let mut buffer = vec![0; len];
        repr.emit(&mut Frame::new_unchecked(&mut buffer[..]));
    });
}
