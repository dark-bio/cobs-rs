// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

use std::process::Command;

fn main() {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Failed to execute rustc");

    let version = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 from rustc")
        .trim()
        .to_string();

    println!("cargo::rerun-if-env-changed=RUSTC");
    println!("cargo::rustc-env=RUSTC_VERSION={}", version);
}
