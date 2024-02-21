use crate::phy::Device;

pub struct CsmaDevice<D: Device> {
    device: D,
}

impl<D: Device> CsmaDevice<D> {
    pub fn new(mut device: D) -> Self {
        device.disable();
        Self { device }
    }
}
