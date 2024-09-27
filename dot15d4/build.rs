use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;

fn main() {
    // (Variable, Type, Default value)
    let mut configs: HashMap<&str, (&str, &str)> = HashMap::from([
        ("MAC_MIN_BE", ("u16", "0")),
        ("MAC_MAX_BE", ("u16", "8")),
        ("MAC_MAX_CSMA_BACKOFFS", ("u16", "16")),
        (
            "MAC_UNIT_BACKOFF_DURATION",
            (
                "Duration",
                "Duration::from_us((UNIT_BACKOFF_PERIOD * SYMBOL_RATE_INV_US) as i64)",
            ),
        ),
        ("MAC_MAX_FRAME_RETIES", ("u16", "3")),
        (
            "CSMA_INTER_FRAME_TIME",
            ("Duration", "Duration::from_us(1000)"),
        ),
        ("MAC_AIFS_PERIOD", ("Duration", "Duration::from_us(1000)")),
        ("MAC_SIFS_PERIOD", ("Duration", "Duration::from_us(1000)")),
        ("MAC_LIFS_PERIOD", ("Duration", "Duration::from_us(10_000)")),
        ("MAC_PAN_ID", ("u16", "0xffff")),
        ("MAC_IMPLICIT_BROADCAST", ("bool", "false")),
    ]);

    // Make sure we get rerun if needed
    println!("cargo:rerun-if-changed=build.rs");
    for name in configs.keys() {
        println!("cargo:rerun-if-env-changed=DOT15D4_{name}");
    }

    // Collect environment variables
    let mut data = String::new();
    // Write preamble
    writeln!(data, "use crate::time::Duration;").unwrap();
    writeln!(
        data,
        "use crate::csma::{{SYMBOL_RATE_INV_US, UNIT_BACKOFF_PERIOD}};"
    )
    .unwrap();

    for (var, value) in std::env::vars() {
        if let Some(name) = var.strip_prefix("DOT15D4_") {
            // discard from hashmap as a way of consuming the setting
            let Some((_, (ty, _))) = configs.remove_entry(name) else {
                panic!("Wrong configuration name {name}");
            };

            // write to file
            writeln!(data, "pub const {name}: {ty} = {value};").unwrap();
        }
    }

    // Take the remaining configs and write the default value to the file
    for (name, (ty, value)) in configs.iter() {
        writeln!(data, "pub const {name}: {ty} = {value};").unwrap();
    }

    // Now that we have the code of the configuration, actually write it to a file
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("config.rs");
    std::fs::write(out_file, data).unwrap();
}
