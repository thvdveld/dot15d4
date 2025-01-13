use crate::*;

#[test]
fn parse_data_frame() {
    let frame = [
        0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12, 0x00,
        0x2b, 0x00, 0x00, 0x00,
    ];

    let frame = DataFrame::new(&frame).unwrap();

    test!(
        frame.frame_control().frame_type() => FrameType::Data,
        frame.frame_control().security_enabled() => false,
        frame.frame_control().frame_pending() => false,
        frame.frame_control().ack_request() => false,
        frame.frame_control().pan_id_compression() => true,
        frame.frame_control().sequence_number_suppression() => false,
        frame.frame_control().information_elements_present() => false,
        frame.frame_control().dst_addressing_mode() => AddressingMode::Short,
        frame.frame_control().frame_version() => FrameVersion::Ieee802154_2006,
        frame.frame_control().src_addressing_mode() => AddressingMode::Extended,
        frame.sequence_number() => Some(1),
        frame.addressing().unwrap().dst_pan_id() => Some(0xabcd),
        frame.addressing().unwrap().dst_address() => Some(Address::BROADCAST),
        frame.addressing().unwrap().src_pan_id() => None,
        frame.addressing().unwrap().src_address() => Some(Address::Extended([0x00, 0x12, 0x4b, 0x00, 0x14, 0xb5, 0xd9, 0xc7])),
        frame.information_elements() => None,
        frame.payload() => Some(&[0x2b, 0x00, 0x00, 0x00][..]),
    );
}
