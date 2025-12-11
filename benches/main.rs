// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group};
use darkbio_cobs::{decode, decode_buffer, decode_unsafe, encode, encode_buffer, encode_unsafe};
use rand::Rng;
use sysinfo::System;

/// Benchmarks the encoding speed of the safe COBS encoder.
fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut buffer = vec![0u8; encode_buffer(size)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                encode(data, &mut buffer).unwrap();
            });
        });
    }
    group.finish();
}

/// Benchmarks the encoding speed of the unsafe COBS encoder.
fn bench_encode_unsafe(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_unsafe");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut buffer = vec![0u8; encode_buffer(size)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                encode_unsafe(data, &mut buffer);
            });
        });
    }
    group.finish();
}

/// Benchmarks the decoding speed of the safe COBS decoder.
fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut encoded = vec![0u8; encode_buffer(size)];

        let len = encode(&data, &mut encoded).unwrap();
        encoded.truncate(len);

        let mut buffer = vec![0u8; decode_buffer(encoded.len())];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &encoded, |b, encoded| {
            b.iter(|| {
                decode(encoded, &mut buffer).unwrap();
            });
        });
    }
    group.finish();
}

/// Benchmarks the decoding speed of the unsafe COBS decoder.
fn bench_decode_unsafe(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_unsafe");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut encoded = vec![0u8; encode_buffer(size)];

        let len = encode(&data, &mut encoded).unwrap();
        encoded.truncate(len);

        let mut buffer = vec![0u8; decode_buffer(encoded.len())];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &encoded, |b, encoded| {
            b.iter(|| {
                decode_unsafe(encoded, &mut buffer).unwrap();
            });
        });
    }
    group.finish();
}

/// Benchmarks the encoding speed of the jamesmunns/cobs encoder.
fn bench_jamesmunns_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("jamesmunns/encode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut buffer = vec![0u8; cobs::max_encoding_length(size)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                cobs::encode(data, &mut buffer);
            });
        });
    }
    group.finish();
}

/// Benchmarks the decoding speed of the jamesmunns/cobs decoder.
fn bench_jamesmunns_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("jamesmunns/decode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut encoded = vec![0u8; cobs::max_encoding_length(size)];

        let len = cobs::encode(&data, &mut encoded);
        encoded.truncate(len);

        let mut buffer = vec![0u8; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &encoded, |b, encoded| {
            b.iter(|| {
                cobs::decode(encoded, &mut buffer).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_encode_unsafe,
    bench_decode_unsafe,
    bench_jamesmunns_encode,
    bench_jamesmunns_decode
);

fn main() {
    print_system_infos();
    benches();
    Criterion::default().configure_from_args().final_summary();
}

/// Prints a collection of system hardware, software and runtime infos so that
/// benchmarks originating from different people can be meaningfully compared.
fn print_system_infos() {
    // Print operating system infos
    println!("Benchmark Environment:");
    println!(
        "  OS:        {} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "".to_string())
    );
    println!(
        "  Kernel:    {}",
        System::kernel_version().unwrap_or_else(|| "Unknown".to_string())
    );
    println!("  Arch:      {}", std::env::consts::ARCH);

    // Print hardware infos
    let sys = System::new_all();
    let cpus = sys.cpus();
    if let Some(cpu) = cpus.first() {
        println!("  CPU:       {}", cpu.brand().trim());
    }
    println!("  Cores:     {}", cpus.len());
    println!(
        "  Memory:    {:.2} GB / {:.2} GB",
        sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
    );

    // Print Rust runtime infos
    #[cfg(debug_assertions)]
    println!("  Build:     debug");
    #[cfg(not(debug_assertions))]
    println!("  Build:     release");

    println!("  Rustc:     {}", env!("RUSTC_VERSION"));
    println!();
}
