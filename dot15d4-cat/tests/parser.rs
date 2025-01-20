use dot15d4_cat::FrameParser;

use strip_ansi_escapes::strip;

#[test]
fn enhanced_beacon() {
    let input = "40ebcdabffff0100010001000100003f1188061a0e0000000000011c0001c800011b00";
    let output = String::from_utf8(strip(FrameParser::parse_hex(input).unwrap())).unwrap();
    assert_eq!(
        output,
        "Frame Control
  frame type: Enhanced Beacon
  security: 0
  frame pending: 0
  ack request: 0
  pan id compression: 1
  sequence number suppression: 1
  information elements present: 1
  dst addressing mode: Short
  src addressing mode: Extended
  frame version: 2 (Ieee802154_2020)
Addressing
  dst pan id: abcd
  dst addr: ff:ff (broadcast)
  src addr: 00:01:00:01:00:01:00:01
Information Elements
  Header Information Elements
    HeaderTermination1
  Payload Information Elements
    MLME
      TschSynchronization
        ASN: 14, join metric: 0
      TschTimeslot
        slot ID: 0
      ChannelHopping
        sequence ID: 0
      TschSlotframeAndLink
        #slotframes: 0
"
    );
}

#[test]
fn enhanced_beacon_with_slotframes() {
    let input = "40ebcdabffff0100010001000100003f3788061a110000000000191c01080780004808fc032003e80398089001c0006009a010102701c8000f1b010011000200000100060100020007";
    let output = String::from_utf8(strip(FrameParser::parse_hex(input).unwrap())).unwrap();
    assert_eq!(
        output,
        "Frame Control
  frame type: Enhanced Beacon
  security: 0
  frame pending: 0
  ack request: 0
  pan id compression: 1
  sequence number suppression: 1
  information elements present: 1
  dst addressing mode: Short
  src addressing mode: Extended
  frame version: 2 (Ieee802154_2020)
Addressing
  dst pan id: abcd
  dst addr: ff:ff (broadcast)
  src addr: 00:01:00:01:00:01:00:01
Information Elements
  Header Information Elements
    HeaderTermination1
  Payload Information Elements
    MLME
      TschSynchronization
        ASN: 17, join metric: 0
      TschTimeslot
        slot ID: 1
        cca_offset: 1.80ms
        cca: 0.13ms
        tx offset: 2.12ms
        rx offset: 1.02ms
        tx ack delay: 1.00ms
        rx ack delay: 0.80ms
        rx wait: 2.20ms
        ack wait: 0.40ms
        rx/tx: 0.19ms
        max ack: 2.40ms
        max tx: 4.26ms
        timeslot length: 10.00ms
      ChannelHopping
        sequence ID: 0
      TschSlotframeAndLink
        #slotframes: 1
        Slotframe Handle: 0, #links: 2
          Timeslot: 0, Channel Offset: 1, Link Options: Rx | Shared
          Timeslot: 1, Channel Offset: 2, Link Options: Tx | Rx | Shared
"
    );
}

#[test]
fn enhanced_ack() {
    let input = "022e37cdab0200020002000200020fe18f";
    let output = String::from_utf8(strip(FrameParser::parse_hex(input).unwrap())).unwrap();
    assert_eq!(
        output,
        "Frame Control
  frame type: Enhanced Ack
  security: 0
  frame pending: 0
  ack request: 0
  pan id compression: 0
  sequence number suppression: 0
  information elements present: 1
  dst addressing mode: Extended
  src addressing mode: Absent
  frame version: 2 (Ieee802154_2020)
Sequence Number
  sequence number: 55
Addressing
  dst pan id: abcd
  dst addr: 00:02:00:02:00:02:00:02
  src addr: absent
Information Elements
  Header Information Elements
    TimeCorrection
      -0.03ms, nack: 1
Payload
  []
"
    );
}

#[test]
fn data_frame() {
    let input = "41d801cdabffffc7d9b514004b12002b000000";
    let output = String::from_utf8(strip(FrameParser::parse_hex(input).unwrap())).unwrap();
    assert_eq!(
        output,
        "Frame Control
  frame type: Data
  security: 0
  frame pending: 0
  ack request: 0
  pan id compression: 1
  sequence number suppression: 0
  information elements present: 0
  dst addressing mode: Short
  src addressing mode: Extended
  frame version: 1 (Ieee802154_2006)
Sequence Number
  sequence number: 1
Addressing
  dst pan id: abcd
  dst addr: ff:ff (broadcast)
  src addr: 00:12:4b:00:14:b5:d9:c7
Payload
  [2b, 0, 0, 0]
"
    );
}
