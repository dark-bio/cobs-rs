// cobs-rs: fast cobs encoder and decoder
// Copyright 2025 Dark Bio AG. All rights reserved.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use darkbio_cobs::{decode, encode, encode_buffer};
use rand::Rng;

// Benchmarks the encoding speed of the COBS library.
fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut buffer = vec![0u8; encode_buffer(size)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                encode(data, &mut buffer);
            });
        });
    }
    group.finish();
}

// Benchmarks the decoding speed of the COBS library.
fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for size in [16, 256, 4096, 65536, 262144, 1048576, 4194304] {
        let data: Vec<u8> = rand::rng().random_iter().take(size).collect();
        let mut encoded = vec![0u8; encode_buffer(size)];

        let len = encode(&data, &mut encoded);
        encoded.truncate(len);

        let mut buffer = vec![0u8; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &encoded, |b, encoded| {
            b.iter(|| {
                decode(encoded, &mut buffer).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode,);
criterion_main!(benches);
