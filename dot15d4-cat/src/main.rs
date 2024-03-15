use clap::Parser;
use colored::*;
use dot15d4_frame::*;

struct Writer {
    indent: usize,
}

impl Writer {
    fn new(indent: usize) -> Self {
        Self { indent }
    }

    fn increase_indent(&mut self) {
        self.indent += 2;
    }

    fn decrease_indent(&mut self) {
        self.indent -= 2;
    }

    fn write(&self, s: String) {
        print!("{}", " ".repeat(self.indent));
        print!("{}", s);
    }

    fn writeln(&self, s: String) {
        self.write(s);
        println!();
    }
}

/// `cat` for IEEE 802.15.4 frames.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The IEEE 802.15.4 frame to parse.
    #[clap(value_parser(clap::builder::NonEmptyStringValueParser::new()))]
    input: String,
}

fn main() {
    let args = Args::parse();
    let data = hex::decode(args.input).unwrap();
    let frame = Frame::new(&data[..]).unwrap();

    let mut w = Writer::new(0);

    let fc = frame.frame_control();

    // -----------------------------------------------------------------
    // Frame Control
    // -----------------------------------------------------------------
    w.writeln("Frame Control".underline().bold().to_string());
    w.increase_indent();
    w.writeln(format!(
        "{}: {}{:?}",
        "frame type".bold(),
        if fc.frame_version() == FrameVersion::Ieee802154_2020
            && fc.frame_type() == FrameType::Beacon
        {
            "Enhanced "
        } else {
            ""
        },
        fc.frame_type()
    ));
    w.writeln(format!(
        "{}: {}",
        "security".bold(),
        fc.security_enabled() as usize
    ));
    w.writeln(format!(
        "{}: {}",
        "frame pending".bold(),
        fc.frame_pending() as usize
    ));
    w.writeln(format!(
        "{}: {}",
        "ack request".bold(),
        fc.ack_request() as usize
    ));
    w.writeln(format!(
        "{}: {}",
        "pan id compression".bold(),
        fc.pan_id_compression() as usize
    ));
    w.writeln(format!(
        "{}: {}",
        "sequence number suppression".bold(),
        fc.sequence_number_suppression() as usize
    ));
    w.writeln(format!(
        "{}: {}",
        "information elements present".bold(),
        fc.information_elements_present() as usize
    ));
    w.writeln(format!(
        "{}: {:?}",
        "dst addressing mode".bold(),
        fc.dst_addressing_mode()
    ));
    w.writeln(format!(
        "{}: {:?}",
        "src addressing mode".bold(),
        fc.src_addressing_mode()
    ));
    w.writeln(format!(
        "{}: {} ({:?})",
        "frame version".bold(),
        fc.frame_version() as usize,
        fc.frame_version()
    ));
    w.decrease_indent();

    // -----------------------------------------------------------------
    // Sequence Number
    // -----------------------------------------------------------------
    if let Some(seq) = frame.sequence_number() {
        w.writeln(format!("{}", "Sequence Number".underline().bold()));
        w.increase_indent();
        w.writeln(format!("{}: {}", "sequence number".bold(), seq));
        w.decrease_indent();
    }

    // -----------------------------------------------------------------
    // Addressing
    // -----------------------------------------------------------------
    if let Some(addr) = frame.addressing() {
        w.writeln(format!("{}", "Addressing".underline().bold()));
        w.increase_indent();

        if let Some(dst_pan_id) = addr.dst_pan_id() {
            w.writeln(format!("{}: {:x}", "dst pan id".bold(), dst_pan_id));
        }

        if let Some(dst_addr) = addr.dst_address() {
            w.writeln(format!(
                "{}: {}{}",
                "dst addr".bold(),
                dst_addr,
                if dst_addr.is_broadcast() {
                    " (broadcast)"
                } else {
                    ""
                }
            ));
        }

        if let Some(src_pan_id) = addr.src_pan_id() {
            w.writeln(format!("{}: {:x}", "src pan id".bold(), src_pan_id));
        }

        if let Some(src_addr) = addr.src_address() {
            w.writeln(format!(
                "{}: {}{}",
                "src addr".bold(),
                src_addr,
                if src_addr.is_broadcast() {
                    " (broadcast)"
                } else {
                    ""
                }
            ));
        }
        w.decrease_indent();
    }

    // -----------------------------------------------------------------
    // Auxiliary Security Header
    // -----------------------------------------------------------------
    if frame.auxiliary_security_header().is_some() {
        w.writeln(format!(
            "{}",
            "Auxiliary Security Header".underline().bold()
        ));
        w.increase_indent();
        w.writeln("unimplementec".to_string());
        w.decrease_indent();
    }

    // -----------------------------------------------------------------
    // Information Elements
    // -----------------------------------------------------------------
    if let Some(ie) = frame.information_elements() {
        w.writeln(format!("{}", "Information Elements".underline().bold()));

        // -------------------------------------------------------------
        // Header Information Elements
        // -------------------------------------------------------------
        let headers: Vec<HeaderInformationElement<&[u8]>> =
            ie.header_information_elements().collect();
        if !headers.is_empty() {
            w.increase_indent();
            w.writeln(format!("{}", "Header Information Elements".italic()));

            for header in headers {
                w.increase_indent();
                let id = header.element_id();
                if matches!(
                    id,
                    HeaderElementId::HeaderTermination1 | HeaderElementId::HeaderTermination2
                ) {
                    w.writeln(format!("{}", format!("{:?}", header.element_id()).bold()));
                } else {
                    w.writeln(format!("{}", format!("{:?}", header.element_id()).bold()));

                    w.increase_indent();
                    match id {
                        HeaderElementId::TimeCorrection => {
                            if let Ok(tc) = TimeCorrection::new(header.content()) {
                                w.writeln(format!("{tc}"));
                            } else {
                                w.writeln("invalid".to_string());
                            }
                        }
                        _ => w.writeln("unimplemented".to_string()),
                    }
                    w.decrease_indent();
                }
                w.decrease_indent();
            }
            w.decrease_indent();
        }

        // -------------------------------------------------------------
        // Payload Information Elements
        // -------------------------------------------------------------
        let payloads: Vec<PayloadInformationElement<&[u8]>> =
            ie.payload_information_elements().collect();
        if !payloads.is_empty() {
            w.increase_indent();
            w.writeln(format!("{}", "Payload Information Elements".italic()));

            for payload in payloads {
                w.increase_indent();
                match payload.group_id() {
                    PayloadGroupId::Mlme => {
                        w.writeln("MLME".to_string());

                        for nested in payload.nested_information_elements() {
                            w.increase_indent();
                            w.writeln(format!(
                                "{}",
                                match nested.sub_id() {
                                    NestedSubId::Short(id) => format!("{id:?}").bold(),
                                    NestedSubId::Long(id) => format!("{id:?}").bold(),
                                }
                            ));

                            w.increase_indent();
                            match nested.sub_id() {
                                NestedSubId::Short(NestedSubIdShort::TschSynchronization) => {
                                    if let Ok(sync) = TschSynchronization::new(nested.content()) {
                                        w.writeln(format!("{sync}"));
                                    } else {
                                        w.writeln("invalid".to_string());
                                    }
                                }
                                NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
                                    if let Ok(timeslot) = TschTimeslot::new(nested.content()) {
                                        w.writeln(format!("{timeslot}"));
                                    } else {
                                        w.writeln("invalid".to_string());
                                    }
                                }
                                NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink) => {
                                    if let Ok(slotframe_and_link) =
                                        TschSlotframeAndLink::new(nested.content())
                                    {
                                        w.writeln(format!("{slotframe_and_link}"));
                                    } else {
                                        w.writeln("invalid".to_string());
                                    }
                                }
                                NestedSubId::Long(NestedSubIdLong::ChannelHopping) => {
                                    if let Ok(channel_hopping) =
                                        ChannelHopping::new(nested.content())
                                    {
                                        w.writeln(format!("{channel_hopping}"));
                                    } else {
                                        w.writeln("invalid".to_string());
                                    }
                                }
                                _ => w.writeln("unimplemented".to_string()),
                            }
                            w.decrease_indent();
                            w.decrease_indent();
                        }
                    }
                    id => w.writeln(format!("{}: unimplemented", format!("{:?}", id).bold())),
                }

                w.decrease_indent();
            }

            w.decrease_indent();
        }
    }

    // -----------------------------------------------------------------
    // Payload
    // -----------------------------------------------------------------
    if let Some(payload) = frame.payload() {
        w.writeln(format!("{}", "Payload".underline().bold()));
        w.increase_indent();
        w.writeln(format!("{:x?}", payload));
    }
}
