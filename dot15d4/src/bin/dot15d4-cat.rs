use clap::Parser;
use colored::*;
use dot15d4::frame::*;

/// `cat` for IEEE 802.15.4 frames.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The IEEE 802.15.4 frame to parse.
    input: String,
}

fn main() {
    let args = Args::parse();
    let data = hex::decode(args.input).unwrap();
    let frame = Frame::new(&data[..]).unwrap();

    let fc = frame.frame_control();
    println!("{}", "Frame Control".underline().bold());
    println!(
        "  {}: {}{:?}",
        "frame type".bold(),
        if fc.frame_version() == FrameVersion::Ieee802154_2020
            && fc.frame_type() == FrameType::Beacon
        {
            "Enhanced "
        } else {
            ""
        },
        fc.frame_type()
    );
    println!(
        "  {}: {}",
        "security".bold(),
        fc.security_enabled() as usize
    );
    println!(
        "  {}: {}",
        "frame pending".bold(),
        fc.frame_pending() as usize
    );
    println!("  {}: {}", "ack request".bold(), fc.ack_request() as usize);
    println!(
        "  {}: {}",
        "pan id compression".bold(),
        fc.pan_id_compression() as usize
    );
    println!(
        "  {}: {}",
        "sequence number suppression".bold(),
        fc.sequence_number_suppression() as usize
    );
    println!(
        "  {}: {}",
        "information elements present".bold(),
        fc.information_elements_present() as usize
    );
    println!(
        "  {}: {:?}",
        "dst addressing mode".bold(),
        fc.dst_addressing_mode()
    );
    println!(
        "  {}: {:?}",
        "src addressing mode".bold(),
        fc.src_addressing_mode()
    );
    println!(
        "  {}: {} ({:?})",
        "frame version".bold(),
        fc.frame_version() as usize,
        fc.frame_version()
    );

    if let Some(seq) = frame.sequence_number() {
        println!("{}", "Sequence Number".underline().bold());
        println!("  {}: {}", "sequence number".bold(), seq);
    }

    if let Some(addr) = frame.addressing() {
        println!("{}", "Addressing".underline().bold());

        if let Some(dst_pan_id) = addr.dst_pan_id(&fc) {
            println!("  {}: {:x}", "dst pan id".bold(), dst_pan_id);
        }

        if let Some(dst_addr) = addr.dst_address(&fc) {
            println!(
                "  {}: {}{}",
                "dst addr".bold(),
                dst_addr,
                if dst_addr.is_broadcast() {
                    " (broadcast)"
                } else {
                    ""
                }
            );
        }

        if let Some(src_pan_id) = addr.src_pan_id(&fc) {
            println!("  {}: {:x}", "src pan id".bold(), src_pan_id);
        }

        if let Some(src_addr) = addr.src_address(&fc) {
            println!(
                "  {}: {}{}",
                "src addr".bold(),
                src_addr,
                if src_addr.is_broadcast() {
                    " (broadcast)"
                } else {
                    ""
                }
            );
        }

        if let Some(_) = frame.auxiliary_security_header() {
            println!("{}", "Auxiliary Security Header".underline().bold());
            println!("  unimplementec");
        }

        if let Some(ie) = frame.information_elements() {
            println!("{}", "Information Elements".underline().bold());

            let headers: Vec<HeaderInformationElement<&[u8]>> =
                ie.header_information_elements().collect();
            if !headers.is_empty() {
                println!("  {}", "Header Information Elements".italic());

                for header in headers {
                    let id = header.element_id();
                    if matches!(
                        id,
                        HeaderElementId::HeaderTermination1 | HeaderElementId::HeaderTermination2
                    ) {
                        println!("    {}", format!("{:?}", header.element_id()).bold());
                    } else {
                        println!("    {}", format!("{:?}", header.element_id()).bold(),);

                        match id {
                            HeaderElementId::TimeCorrection => {
                                println!("      {}", TimeCorrection::new(header.content()));
                            }
                            _ => println!("        unimplemented"),
                        }
                    }
                }
            }

            let payloads: Vec<PayloadInformationElement<&[u8]>> =
                ie.payload_information_elements().collect();
            if !payloads.is_empty() {
                println!("  {}", "Payload Information Elements".italic());

                for payload in payloads {
                    match payload.group_id() {
                        PayloadGroupId::Mlme => {
                            println!("    {}", "Mlme");
                            for nested in payload.nested_information_elements() {
                                println!(
                                    "      {}",
                                    match nested.sub_id() {
                                        NestedSubId::Short(id) => format!("{:?}", id).bold(),
                                        NestedSubId::Long(id) => format!("{:?}", id).bold(),
                                    }
                                );

                                match nested.sub_id() {
                                    NestedSubId::Short(NestedSubIdShort::TschSynchronization) => {
                                        println!(
                                            "        {}",
                                            TschSynchronization::new(nested.content())
                                        );
                                    }
                                    NestedSubId::Short(NestedSubIdShort::TschTimeslot) => {
                                        println!("        {}", TschTimeslot::new(nested.content()));
                                    }
                                    NestedSubId::Short(NestedSubIdShort::TschSlotframeAndLink) => {
                                        println!(
                                            "        {}",
                                            TschSlotframeAndLink::new(nested.content())
                                        );
                                    }
                                    NestedSubId::Long(NestedSubIdLong::ChannelHopping) => {
                                        println!(
                                            "        {}",
                                            ChannelHopping::new(nested.content())
                                        );
                                    }
                                    _ => println!("        unimplemented"),
                                }
                            }
                        }
                        id => println!("      {}: unimplemented", format!("{:?}", id).bold()),
                    }
                }
            }
        }

        if let Some(payload) = frame.payload() {
            println!("{}", "Payload".underline().bold());
            println!("  {:x?}", payload);
        }
    }
}
