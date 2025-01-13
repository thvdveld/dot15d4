use crate::*;

use crate::frames::ack::*;

#[test]
fn parse_imm_ack() {
    let frame = [0x02, 0x10, 0x01];

    let frame = AckFrame::new(&frame).unwrap();

    let fc = frame.frame_control();
    assert_eq!(fc.frame_type(), FrameType::Ack);
    assert!(!fc.security_enabled());
    assert!(!fc.frame_pending());
    assert!(!fc.ack_request());
    assert!(!fc.pan_id_compression());
    assert!(!fc.sequence_number_suppression());
    assert!(!fc.information_elements_present());
    assert!(fc.dst_addressing_mode() == AddressingMode::Absent);
    assert!(fc.frame_version() == FrameVersion::Ieee802154_2006);
    assert!(fc.src_addressing_mode() == AddressingMode::Absent);
    assert!(frame.sequence_number() == 1);
}

#[test]
fn parse_enhanced_ack() {
    let frame = [
        0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x0f,
        0xe1, 0x8f,
    ];

    let frame = EnhancedAckFrame::new(&frame).unwrap();

    let fc = frame.frame_control();
    assert_eq!(fc.frame_type(), FrameType::Ack);
    assert!(!fc.security_enabled());
    assert!(!fc.frame_pending());
    assert!(!fc.ack_request());
    assert!(!fc.pan_id_compression());
    assert!(!fc.sequence_number_suppression());
    assert!(fc.information_elements_present());
    assert!(fc.dst_addressing_mode() == AddressingMode::Extended);
    assert!(fc.frame_version() == FrameVersion::Ieee802154_2020);
    assert!(fc.src_addressing_mode() == AddressingMode::Absent);

    assert!(frame.sequence_number() == Some(55));

    let addressing = frame.addressing().unwrap();
    assert_eq!(addressing.dst_pan_id(), Some(0xabcd));
    assert_eq!(
        addressing.dst_address(),
        Some(Address::Extended([
            0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02
        ]))
    );
    assert_eq!(addressing.src_pan_id(), None);
    assert_eq!(addressing.src_address(), Some(Address::Absent));

    let ie = frame.information_elements().unwrap();
    let mut headers = ie.header_information_elements();

    let time_correction = headers.next().unwrap();
    assert_eq!(
        time_correction.element_id(),
        HeaderElementId::TimeCorrection
    );

    let time_correction = TimeCorrection::new(time_correction.content()).unwrap();
    assert_eq!(time_correction.len(), 2);
    assert_eq!(
        time_correction.time_correction(),
        crate::time::Duration::from_us(-31)
    );
    assert!(time_correction.nack());
}
