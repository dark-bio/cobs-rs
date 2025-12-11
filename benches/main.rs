// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use darkbio_cobs::{decode, decode_buffer, decode_unsafe, encode, encode_buffer, encode_unsafe};
use rand::Rng;

// Benchmarks the encoding speed of the safe COBS encoder.
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

// Benchmarks the encoding speed of the unsafe COBS encoder.
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

// Benchmarks the decoding speed of the safe COBS decoder.
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

// Benchmarks the decoding speed of the unsafe COBS decoder.
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

criterion_group!(
    benches,
    bench_encode,
    bench_encode_unsafe,
    bench_decode,
    bench_decode_unsafe
);
criterion_main!(benches);
