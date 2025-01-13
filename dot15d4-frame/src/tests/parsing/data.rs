use crate::*;

#[test]
fn parse_data_frame() {
    let frame = [
        0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12, 0x00,
        0x2b, 0x00, 0x00, 0x00,
    ];

    let frame = DataFrame::new(&frame).unwrap();

    let fc = frame.frame_control();
    assert_eq!(fc.frame_type(), FrameType::Data);
    assert!(!fc.security_enabled());
    assert!(!fc.frame_pending());
    assert!(!fc.ack_request());
    assert!(fc.pan_id_compression());

    assert!(fc.dst_addressing_mode() == AddressingMode::Short);
    assert!(fc.frame_version() == FrameVersion::Ieee802154_2006);
    assert!(fc.src_addressing_mode() == AddressingMode::Extended);

    assert!(frame.sequence_number() == Some(1));

    let addressing = frame.addressing().unwrap();
    assert_eq!(addressing.dst_pan_id(), Some(0xabcd));
    assert_eq!(addressing.dst_address(), Some(Address::BROADCAST));
    assert_eq!(addressing.src_pan_id(), None);
    assert_eq!(
        addressing.src_address(),
        Some(Address::Extended([
            0x00, 0x12, 0x4b, 0x00, 0x14, 0xb5, 0xd9, 0xc7
        ]))
    );

    assert!(frame.information_elements().is_none());

    assert!(frame.payload() == Some(&[0x2b, 0x00, 0x00, 0x00][..]));
}
