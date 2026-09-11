// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

use std::env;
use std::process::Command;

fn main() {
    // Cargo names the compiler it drives, so the report matches the build
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
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
