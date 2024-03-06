use super::*;

#[test]
fn parse_imm_ack() {
    let frame = [0x02, 0x10, 0x01];

    let frame = Frame::new(&frame).unwrap();

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
    assert!(frame.sequence_number() == Some(1));
    assert!(frame.addressing().is_none());
    assert!(frame.information_elements().is_none());
    assert!(frame.payload().is_none());
}

#[test]
fn emit_imm_ack() {
    let imm_ack = FrameBuilder::new_imm_ack(1).finalize().unwrap();

    let mut buffer = vec![0; imm_ack.buffer_len()];
    imm_ack.emit(&mut Frame::new_unchecked(&mut buffer[..]));

    assert_eq!(buffer, [0x02, 0x10, 0x01]);
}

#[test]
fn parse_ack_frame() {
    let frame = [
        0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x0f,
        0xe1, 0x8f,
    ];

    let frame = Frame::new(&frame).unwrap();

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
    assert_eq!(addressing.dst_pan_id(&fc), Some(0xabcd));
    assert_eq!(
        addressing.dst_address(&fc),
        Some(Address::Extended([
            0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02
        ]))
    );
    assert_eq!(addressing.src_pan_id(&fc), None);
    assert_eq!(addressing.src_address(&fc), Some(Address::Absent));

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
                time_correction: Duration::from_us(-31),
                nack: true,
            },
        ))
        .finalize()
        .unwrap();

    let mut buffer = vec![0; frame.buffer_len()];
    frame.emit(&mut Frame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x02, 0x2e, 0x37, 0xcd, 0xab, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02,
            0x0f, 0xe1, 0x8f,
        ]
    );
}

#[test]
fn parse_data_frame() {
    let frame = [
        0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12, 0x00,
        0x2b, 0x00, 0x00, 0x00,
    ];

    let frame = Frame::new(&frame).unwrap();

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
    assert_eq!(addressing.dst_pan_id(&fc), Some(0xabcd));
    assert_eq!(addressing.dst_address(&fc), Some(Address::BROADCAST));
    assert_eq!(addressing.src_pan_id(&fc), None);
    assert_eq!(
        addressing.src_address(&fc),
        Some(Address::Extended([
            0x00, 0x12, 0x4b, 0x00, 0x14, 0xb5, 0xd9, 0xc7
        ]))
    );

    assert!(frame.information_elements().is_none());

    assert!(frame.payload() == Some(&[0x2b, 0x00, 0x00, 0x00][..]));
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

    frame.emit(&mut Frame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x41, 0xd8, 0x01, 0xcd, 0xab, 0xff, 0xff, 0xc7, 0xd9, 0xb5, 0x14, 0x00, 0x4b, 0x12,
            0x00, 0x2b, 0x00, 0x00, 0x00,
        ]
    );
}

#[test]
fn parse_enhanced_beacon() {
    let frame: [u8; 35] = [
        0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c, 0x00, 0x01,
        0xc8, 0x00, 0x01, 0x1b, 0x00,
    ];

    let frame = Frame::new(&frame).unwrap();
    let fc = frame.frame_control();
    assert_eq!(fc.frame_type(), FrameType::Beacon);
    assert!(!fc.security_enabled());
    assert!(!fc.frame_pending());
    assert!(!fc.ack_request());
    assert!(fc.pan_id_compression());
    assert!(fc.sequence_number_suppression());
    assert!(fc.information_elements_present());
    assert_eq!(fc.dst_addressing_mode(), AddressingMode::Short);
    assert_eq!(fc.src_addressing_mode(), AddressingMode::Extended);
    assert_eq!(fc.frame_version(), FrameVersion::Ieee802154_2020);

    let addressing = frame.addressing().unwrap();
    assert_eq!(addressing.dst_pan_id(&fc), Some(0xabcd),);
    assert_eq!(addressing.src_pan_id(&fc), None,);
    assert_eq!(addressing.dst_address(&fc), Some(Address::BROADCAST));
    assert_eq!(
        addressing.src_address(&fc),
        Some(Address::Extended([
            0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01
        ]))
    );
    assert_eq!(addressing.len(&fc), 12);

    let ie = frame.information_elements().unwrap();

    let mut headers = ie.header_information_elements();
    let terminator = headers.next().unwrap();
    assert_eq!(terminator.element_id(), HeaderElementId::HeaderTermination1);
    assert_eq!(terminator.len(), 0);

    assert_eq!(headers.next(), None);

    let mut payloads = ie.payload_information_elements();

    let mlme = payloads.next().unwrap();
    assert_eq!(mlme.group_id(), PayloadGroupId::Mlme);
    assert_eq!(mlme.len() + 2, 19);
    assert_eq!(payloads.next(), None);

    let mut nested_iterator = NestedInformationElementsIterator::new(mlme.content());

    let tsch_sync = nested_iterator.next().unwrap();
    assert_eq!(
        tsch_sync.sub_id(),
        NestedSubId::Short(NestedSubIdShort::TschSynchronization)
    );
    assert_eq!(
        TschSynchronization::new(tsch_sync.content())
            .unwrap()
            .absolute_slot_number(),
        14
    );
    assert_eq!(
        TschSynchronization::new(tsch_sync.content())
            .unwrap()
            .join_metric(),
        0
    );

    let tsch_timeslot = nested_iterator.next().unwrap();
    assert_eq!(
        tsch_timeslot.sub_id(),
        NestedSubId::Short(NestedSubIdShort::TschTimeslot)
    );
    assert_eq!(TschTimeslot::new(tsch_timeslot.content()).unwrap().id(), 0);

    let channel_hopping = nested_iterator.next().unwrap();
    assert_eq!(
        channel_hopping.sub_id(),
        NestedSubId::Long(NestedSubIdLong::ChannelHopping)
    );
    assert_eq!(
        ChannelHopping::new(channel_hopping.content())
            .unwrap()
            .hopping_sequence_id(),
        0
    );

    let slotframe = nested_iterator.next().unwrap();
    assert_eq!(
        slotframe.sub_id(),
        NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink)
    );
    assert_eq!(
        TschSlotframeAndLink::new(slotframe.content())
            .unwrap()
            .number_of_slot_frames(),
        0
    );

    assert_eq!(nested_iterator.next(), None);
    assert!(frame.payload().is_none());
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
            header_information_elements: Vec::new(),
            payload_information_elements: Vec::from_iter([PayloadInformationElementRepr::Mlme(
                Vec::from_iter([
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
                ]),
            )]),
        }),
        payload: None,
    };

    let mut buffer = vec![0; frame.buffer_len()];
    frame.emit(&mut Frame::new_unchecked(&mut buffer[..]));

    assert_eq!(
        buffer,
        [
            0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c,
            0x00, 0x01, 0xc8, 0x00, 0x01, 0x1b, 0x00
        ],
    );
}
