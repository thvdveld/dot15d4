use crate::frames::ack::*;
use crate::time::Duration;
use crate::*;

#[test]
fn parse_imm_ack() {
    let frame = [0x02, 0x10, 0x01];
    let frame = AckFrame::new(&frame).unwrap();

    test!(
        frame.frame_control().frame_type() => FrameType::Ack,
        frame.frame_control().security_enabled() => false,
        frame.frame_control().frame_pending() => false,
        frame.frame_control().ack_request() => false,
        frame.frame_control().pan_id_compression() => false,
        frame.frame_control().sequence_number_suppression() => false,
        frame.frame_control().information_elements_present() => false,
        frame.frame_control().dst_addressing_mode() => AddressingMode::Absent,
        frame.frame_control().frame_version() => FrameVersion::Ieee802154_2006,
        frame.frame_control().src_addressing_mode() => AddressingMode::Absent,
        frame.sequence_number() => 1
    );
}

#[test]
fn parse_enhanced_ack() {
    let frame = [
        0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x0f,
        0xe1, 0x8f,
    ];

    let frame = EnhancedAckFrame::new(&frame).unwrap();

    test!(
        frame.frame_control().frame_type() => FrameType::Ack,
        frame.frame_control().security_enabled() => false,
        frame.frame_control().frame_pending() => false,
        frame.frame_control().ack_request() => false,
        frame.frame_control().pan_id_compression() => false,
        frame.frame_control().sequence_number_suppression() => false,
        frame.frame_control().information_elements_present() => true,
        frame.frame_control().dst_addressing_mode() => AddressingMode::Extended,
        frame.frame_control().frame_version() => FrameVersion::Ieee802154_2020,
        frame.frame_control().src_addressing_mode() => AddressingMode::Absent,
        frame.sequence_number() => Some(55),
        frame.addressing().unwrap().dst_pan_id() => Some(0xabcd),
        frame.addressing().unwrap().dst_address() => Some(Address::Extended([0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02])),
        frame.addressing().unwrap().src_pan_id() => None,
        frame.addressing().unwrap().src_address() => Some(Address::Absent),
        frame.auxiliary_security_header() => None,
    );

    let ie = frame.information_elements().unwrap();
    let mut headers = ie.header_information_elements();

    test_sub_element!(
        headers.next().unwrap(),
        |ie| {
            test!(
                ie.element_id() => HeaderElementId::TimeCorrection,
            );
            TimeCorrection::new(ie.content()).unwrap()
        },
        |time_correction| {
            test!(
                time_correction.len() => 2,
                time_correction.time_correction() => Duration::from_us(-31),
                time_correction.nack() => true
            );
        }
    );
}
