use crate::frames::beacon::*;

use crate::*;

#[test]
fn superframe_specification() {
    let data = [0xff, 0x0f];
    let ie = SuperframeSpecification::new(&data).unwrap();

    test!(
        ie.beacon_order() => BeaconOrder::OnDemand,
        ie.superframe_order() => SuperframeOrder::Inactive,
        ie.final_cap_slot() => 0x0f,
        ie.battery_life_extension() => false,
        ie.pan_coordinator() => false,
        ie.association_permit() => false,
    );
}

#[test]
fn gts_specification() {
    use crate::Address;

    let data = [0b0000_0000];
    let gts = GtsSpecification::new(&data).unwrap();
    test!(
        gts.gts_permit() => false,
        gts.descriptor_count() => 0,
    );

    let data = [0b1000_0000];
    let gts = GtsSpecification::new(&data).unwrap();
    test!(
        gts.gts_permit() => true,
        gts.descriptor_count() => 0,
        gts.slots().next() => None,
    );

    let data = [0x82, 0x01, 0x34, 0x12, 0x11, 0x78, 0x56, 0x14];
    let gts = GtsSpecification::new(&data).unwrap();
    test!(
        gts.gts_permit() => true,
        gts.descriptor_count() => 2,
    );

    let mut slots = gts.slots();
    test_sub_element!(slots.next().unwrap(), |slot| { slot }, |slot| {
        test!(
            slot.short_address() => Address::Short([0x34, 0x12]),
            slot.starting_slot() => 1,
            slot.length() => 1,
            slot.direction() => GtsDirection::Transmit,
        );
    });

    test_sub_element!(slots.next().unwrap(), |slot| { slot }, |slot| {
        test!(
            slot.short_address() => Address::Short([0x78, 0x56]),
            slot.starting_slot() => 4,
            slot.length() => 1,
            slot.direction() => GtsDirection::Receive,
        );
    });

    assert!(slots.next().is_none());
}

#[test]
fn gts_slot() {
    use crate::Address;
    let data = [0xab, 0xcd, 0b0101_1010];
    let slot = GtsSlot::new(&data[..], GtsDirection::Transmit).unwrap();

    test!(
        slot.short_address() => Address::Short([0xab, 0xcd]),
        slot.starting_slot() => 0b1010,
        slot.length() => 0b0101,
        slot.direction() => GtsDirection::Transmit,
    );
}

#[test]
fn parse_enhanced_beacon() {
    let frame: [u8; 35] = [
        0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c, 0x00, 0x01,
        0xc8, 0x00, 0x01, 0x1b, 0x00,
    ];

    let frame = EnhancedBeaconFrame::new(&frame).unwrap();

    test!(
        frame.frame_control().frame_type() => FrameType::Beacon,
        frame.frame_control().security_enabled() => false,
        frame.frame_control().frame_pending() => false,
        frame.frame_control().ack_request() => false,
        frame.frame_control().pan_id_compression() => true,
        frame.frame_control().sequence_number_suppression() => true,
        frame.frame_control().information_elements_present() => true,
        frame.frame_control().dst_addressing_mode() => AddressingMode::Short,
        frame.frame_control().frame_version() => FrameVersion::Ieee802154_2020,
        frame.frame_control().src_addressing_mode() => AddressingMode::Extended,
        frame.sequence_number() => None,
        frame.addressing().unwrap().dst_pan_id() => Some(0xabcd),
        frame.addressing().unwrap().dst_address() => Some(Address::BROADCAST),
        frame.addressing().unwrap().src_pan_id() => None,
        frame.addressing().unwrap().src_address() => Some(Address::Extended([0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01])),
    );

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

    test_sub_element!(
        nested_iterator.next().unwrap(),
        |nested| {
            test!(
                nested.sub_id() => NestedSubId::Short(NestedSubIdShort::TschSynchronization),
            );
            TschSynchronization::new(nested.content()).unwrap()
        },
        |tsch_sync| {
            test!(
                tsch_sync.absolute_slot_number() => 14,
                tsch_sync.join_metric() => 0,
            );
        }
    );

    test_sub_element!(
        nested_iterator.next().unwrap(),
        |nested| {
            test!(
                nested.sub_id() => NestedSubId::Short(NestedSubIdShort::TschTimeslot),
            );
            TschTimeslot::new(nested.content()).unwrap()
        },
        |tsch_timeslot| {
            test!(
                tsch_timeslot.id() => 0,
            );
        }
    );

    test_sub_element!(
        nested_iterator.next().unwrap(),
        |nested| {
            test!(
                nested.sub_id() => NestedSubId::Long(NestedSubIdLong::ChannelHopping),
            );
            ChannelHopping::new(nested.content()).unwrap()
        },
        |channel_hopping| {
            test!(
                channel_hopping.hopping_sequence_id() => 0,
            );
        }
    );

    test_sub_element!(
        nested_iterator.next().unwrap(),
        |nested| {
            test!(
                nested.sub_id() => NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink),
            );
            TschSlotframeAndLink::new(nested.content()).unwrap()
        },
        |slotframe| {
            test!(
                slotframe.number_of_slot_frames() => 0,
            );
        }
    );

    assert_eq!(nested_iterator.next(), None);
    assert!(frame.payload().is_none());
}
