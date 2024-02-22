use dot15d4::time::Instant;
use dot15d4::tsch::TschDevice;

use dot15d4::phy::Device;

use std::collections::VecDeque;

pub struct DummyDevice {
    enabled: bool,
    pub rx_queue: VecDeque<Vec<u8>>,
    pub tx_queue: VecDeque<Vec<u8>>,
}

impl DummyDevice {
    fn new() -> Self {
        Self {
            enabled: false,
            rx_queue: VecDeque::new(),
            tx_queue: VecDeque::new(),
        }
    }
}

impl Device for DummyDevice {
    fn disable(&mut self) {
        self.enabled = false;
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn receive<RX>(&mut self, mut rx: RX)
    where
        RX: FnMut(&[u8], Instant),
    {
        if let Some(frame) = self.rx_queue.pop_front() {
            rx(&frame, Instant::now());
        }
    }

    fn transmit<TX>(&mut self, tx: TX)
    where
        TX: for<'b> Fn(&'b mut [u8]) -> Option<&'b [u8]>,
    {
        let mut buffer = [0u8; 128];
        if let Some(frame) = tx(&mut buffer) {
            self.tx_queue.push_back(frame.to_vec());
        }
    }
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let frame: [u8; 35] = [
        0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x3f, 0x11, 0x88,
        0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c, 0x00, 0x01, 0xc8, 0x00, 0x01, 0x1b, 0x00,
    ];

    let mut dummy = DummyDevice::new();
    dummy.rx_queue.push_back(frame.to_vec());
    let mut tsch = TschDevice::new(&mut dummy);

    for _ in 0..10 {
        tsch.poll(Instant::now());
    }
}
