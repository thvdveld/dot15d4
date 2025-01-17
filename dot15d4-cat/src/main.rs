use clap::Parser;
use dot15d4_cat::FrameParser;

// dot15d4 40ebcdabffff0100010001000100003f1188061a0e0000000000011c0001c800011b00
// dot15d4 022e37cdab0200020002000200020fe18f
// dot15d4 41d801cdabffffc7d9b514004b12002b000000

/// `cat`, but for IEEE 802.15.4 frames.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// The IEEE 802.15.4 frame to parse.
    #[clap(value_parser(clap::builder::NonEmptyStringValueParser::new()))]
    input: String,
}

fn main() {
    let args = Args::parse();
    let data = hex::decode(args.input).unwrap();

    match FrameParser::parse(&data) {
        Ok(parsed) => println!("{}", parsed),
        Err(_) => eprintln!("Failed to parse the frame."),
    }
}
