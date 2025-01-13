use crate::frames::beacon::*;

use crate::*;

#[test]
fn superframe_specification() {
    let data = [0xff, 0x0f];
    let ie = SuperframeSpecification::new(&data).unwrap();
    assert_eq!(ie.beacon_order(), BeaconOrder::OnDemand);
    assert_eq!(ie.superframe_order(), SuperframeOrder::Inactive);
    assert_eq!(ie.final_cap_slot(), 0x0f);
    assert!(!ie.battery_life_extension());
    assert!(!ie.pan_coordinator());
    assert!(!ie.association_permit());
}

#[test]
fn gts_specification() {
    use crate::Address;

    let data = [0b0000_0000];
    let gts = GtsSpecification::new(&data).unwrap();
    assert_eq!(gts.descriptor_count(), 0);
    assert!(!gts.gts_permit());

    let data = [0b1000_0000];
    let gts = GtsSpecification::new(&data).unwrap();
    assert_eq!(gts.descriptor_count(), 0);
    assert!(gts.gts_permit());

    assert!(gts.slots().next().is_none());

    let data = [0x82, 0x01, 0x34, 0x12, 0x11, 0x78, 0x56, 0x14];
    let gts = GtsSpecification::new(&data).unwrap();

    assert!(gts.gts_permit());
    assert_eq!(gts.descriptor_count(), 2);

    let mut slots = gts.slots();
    let s0 = slots.next().unwrap();
    assert_eq!(s0.short_address(), Address::Short([0x34, 0x12]));
    assert_eq!(s0.starting_slot(), 1);
    assert_eq!(s0.length(), 1);
    assert_eq!(s0.direction(), GtsDirection::Transmit);

    let s1 = slots.next().unwrap();
    assert_eq!(s1.short_address(), Address::Short([0x78, 0x56]));
    assert_eq!(s1.starting_slot(), 4);
    assert_eq!(s1.length(), 1);
    assert_eq!(s1.direction(), GtsDirection::Receive);

    assert!(slots.next().is_none());
}

#[test]
fn gts_slot() {
    use crate::Address;
    let data = [0xab, 0xcd, 0b0101_1010];
    let slot = GtsSlot::new(&data[..], GtsDirection::Transmit).unwrap();
    assert_eq!(slot.short_address(), Address::Short([0xab, 0xcd]));
    assert_eq!(slot.starting_slot(), 0b1010);
    assert_eq!(slot.length(), 0b0101);
    assert_eq!(slot.direction(), GtsDirection::Transmit);
}

#[test]
fn parse_enhanced_beacon() {
    let frame: [u8; 35] = [
        0x40, 0xeb, 0xcd, 0xab, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0x3f, 0x11, 0x88, 0x06, 0x1a, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c, 0x00, 0x01,
        0xc8, 0x00, 0x01, 0x1b, 0x00,
    ];

    let frame = EnhancedBeaconFrame::new(&frame).unwrap();
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
    assert_eq!(addressing.dst_pan_id(), Some(0xabcd),);
    assert_eq!(addressing.src_pan_id(), None,);
    assert_eq!(addressing.dst_address(), Some(Address::BROADCAST));
    assert_eq!(
        addressing.src_address(),
        Some(Address::Extended([
            0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01
        ]))
    );
    assert_eq!(addressing.len(), 12);

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
