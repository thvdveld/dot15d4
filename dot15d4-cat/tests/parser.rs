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
