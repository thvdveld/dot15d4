use super::*;

mod parsing;

#[test]
fn emit_imm_ack() {
    let imm_ack = FrameBuilder::new_imm_ack(1).finalize().unwrap();

    let mut buffer = vec![0; imm_ack.buffer_len()];
    imm_ack.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));

    assert_eq!(buffer, [0x02, 0x10, 0x01]);
}

#[test]
fn emit_ack_frame() {
    let frame = FrameBuilder::new_ack()
        .set_sequence_number(55)
        .set_dst_pan_id(0xabcd)
        .set_dst_address(Address::Extended([
            0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02,
        ]))
        .add_header_information_element(HeaderInformationElementRepr::TimeCorrection(
            TimeCorrectionRepr {
                time_correction: crate::time::Duration::from_us(-31),
                nack: true,
            },
        ))
        .finalize()
        .unwrap();

    let mut buffer = vec![0; frame.buffer_len()];
    frame.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02,
            0x0f, 0xe1, 0x8f,
        ]
    );
}

#[test]
fn emit_data_frame() {
    let frame = FrameBuilder::new_data(&[0x2b, 0x00, 0x00, 0x00])
        .set_sequence_number(1)
        .set_dst_pan_id(0xabcd)
        .set_dst_address(Address::BROADCAST)
        .set_src_pan_id(0xabcd)
        .set_src_address(Address::Extended([
            0x00, 0x12, 0x4b, 0x00, 0x14, 0xb5, 0xd9, 0xc7,
        ]))
        .finalize()
        .unwrap();

    let mut buffer = vec![0; frame.buffer_len()];

    frame.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12,
            0x00, 0x2b, 0x00, 0x00, 0x00,
        ]
    );
}

#[test]
fn emit_enhanced_beacon() {
    let frame = FrameRepr {
        frame_control: FrameControlRepr {
            frame_type: FrameType::Beacon,
            security_enabled: false,
            frame_pending: false,
            ack_request: false,
            pan_id_compression: true,
            sequence_number_suppression: true,
            information_elements_present: true,
            dst_addressing_mode: AddressingMode::Short,
            src_addressing_mode: AddressingMode::Extended,
            frame_version: FrameVersion::Ieee802154_2020,
        },
        sequence_number: None,
        addressing_fields: Some(AddressingFieldsRepr {
            dst_pan_id: Some(0xabcd),
            src_pan_id: None,
            dst_address: Some(Address::BROADCAST),
            src_address: Some(Address::Extended([
                0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01,
            ])),
        }),
        information_elements: Some(InformationElementsRepr {
            header_information_elements: heapless::Vec::new(),
            payload_information_elements: heapless::Vec::from_iter([
                PayloadInformationElementRepr::Mlme(heapless::Vec::from_iter([
                    NestedInformationElementRepr::TschSynchronization(TschSynchronizationRepr {
                        absolute_slot_number: 14,
                        join_metric: 0,
                    }),
                    NestedInformationElementRepr::TschTimeslot(TschTimeslotRepr { id: 0 }),
                    NestedInformationElementRepr::ChannelHopping(ChannelHoppingRepr {
                        hopping_sequence_id: 0,
                    }),
                    NestedInformationElementRepr::TschSlotframeAndLink(TschSlotframeAndLinkRepr {
                        number_of_slot_frames: 0,
                    }),
                ])),
            ]),
        }),
        payload: None,
    };

    let mut buffer = vec![0; frame.buffer_len()];
    frame.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c,
            0x00, 0x01, 0xc8, 0x00, 0x01, 0x1b, 0x00
        ],
    );
}

/// https://github.com/thvdveld/dot15d4/issues/29
/// Setting `dst_pan_id` to a different value than `src_pan_id` made the `emit` function panic.
#[test]
fn issue29() {
    let frame = FrameBuilder::new_data(&[0x2b, 0x00, 0x00, 0x00])
        .set_sequence_number(1)
        .set_dst_pan_id(0xabce)
        .set_dst_address(Address::Short([0x02, 0x04]))
        .set_src_pan_id(0xabcd)
        .set_src_address(Address::Extended([
            0x00, 0x12, 0x4b, 0x00, 0x14, 0xb5, 0xd9, 0xc7,
        ]))
        .finalize()
        .unwrap();

    let mut buffer = vec![0; frame.buffer_len()];

    frame.emit(&mut DataFrame::new_unchecked(&mut buffer[..]));

    println!("{:?}", frame);
    println!("packet = {:#04X?}", buffer);
}
