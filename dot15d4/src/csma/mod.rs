use crate::phy::radio::Radio;

pub struct CsmaDevice<R: Radio> {
    device: R,
}

impl<R: Radio> CsmaDevice<R> {
    pub fn new(mut device: R) -> Self {
        Self { device }
    }
}
