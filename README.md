# Fast COBS encoder and decoder

[![](https://img.shields.io/crates/v/darkbio-cobs.svg)](https://crates.io/crates/darkbio-cobs)
[![](https://docs.rs/darkbio-cobs/badge.svg)](https://docs.rs/darkbio-cobs)
[![](https://github.com/dark-bio/cobs-rs/workflows/tests/badge.svg)](https://github.com/dark-bio/cobs-rs/actions/workflows/ci.yml)

This repository is a *fast* implementation of [Consistent Overhead Byte Stuffing (COBS)](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing). It doesn't do much, but it does it fast. Although there might be eventual fixups and feature expansions for streaming codecs, assume the library is "done".

## Performance

You can run the benchmarks to see the performance of the safe versions, unsafe versions and the currently most popular Rust `cobs` package (`v0.5.0`).

```
% cargo bench -- --quiet
```

The report was post-processed to make it denser, but you will see something along the lines of:

```
Benchmark Environment:
  OS:        Darwin 26.1
  Kernel:    25.1.0
  Arch:      aarch64
  CPU:       Apple M2 Max
  Cores:     12
  Memory:    35.85 GB / 64.00 GB
  Build:     release
  Rustc:     rustc 1.91.1 (ed61e7d7e 2025-11-07)

encode/16                   10.035 ns    1.4849 GiB/s
encode/256                  152.35 ns    1.5649 GiB/s
encode/4096                 2.5176 µs    1.5152 GiB/s
encode/65536                40.048 µs    1.5240 GiB/s
encode/262144               161.20 µs    1.5145 GiB/s
encode/1048576              646.75 µs    1.5100 GiB/s
encode/4194304              2.6169 ms    1.4927 GiB/s

decode/16                   6.6440 ns    2.2428 GiB/s
decode/256                  93.850 ns    2.5404 GiB/s
decode/4096                 1.6847 µs    2.2644 GiB/s
decode/65536                25.811 µs    2.3647 GiB/s
decode/262144               103.24 µs    2.3649 GiB/s
decode/1048576              410.68 µs    2.3779 GiB/s
decode/4194304              1.6388 ms    2.3836 GiB/s

encode_unsafe/16            10.030 ns    1.4856 GiB/s
encode_unsafe/256           151.94 ns    1.5692 GiB/s
encode_unsafe/4096          2.4241 µs    1.5736 GiB/s
encode_unsafe/65536         39.285 µs    1.5536 GiB/s
encode_unsafe/262144        160.81 µs    1.5182 GiB/s
encode_unsafe/1048576       633.76 µs    1.5409 GiB/s
encode_unsafe/4194304       2.5506 ms    1.5315 GiB/s

decode_unsafe/16            6.5473 ns    2.2759 GiB/s
decode_unsafe/256           92.253 ns    2.5844 GiB/s
decode_unsafe/4096          1.5776 µs    2.4181 GiB/s
decode_unsafe/65536         25.099 µs    2.4318 GiB/s
decode_unsafe/262144        100.89 µs    2.4200 GiB/s
decode_unsafe/1048576       403.07 µs    2.4228 GiB/s
decode_unsafe/4194304       1.6262 ms    2.4021 GiB/s

jamesmunns/encode/16        10.572 ns    1.4095 GiB/s
jamesmunns/encode/256       155.71 ns    1.5312 GiB/s
jamesmunns/encode/4096      2.6554 µs    1.4366 GiB/s
jamesmunns/encode/65536     43.878 µs    1.3910 GiB/s
jamesmunns/encode/262144    164.73 µs    1.4821 GiB/s
jamesmunns/encode/1048576   670.64 µs    1.4562 GiB/s
jamesmunns/encode/4194304   2.6897 ms    1.4523 GiB/s

jamesmunns/decode/16        18.433 ns    827.79 MiB/s
jamesmunns/decode/256       269.78 ns    904.95 MiB/s
jamesmunns/decode/4096      4.3558 µs    896.80 MiB/s
jamesmunns/decode/65536     69.413 µs    900.41 MiB/s
jamesmunns/decode/262144    280.18 µs    892.28 MiB/s
jamesmunns/decode/1048576   1.1186 ms    893.97 MiB/s
jamesmunns/decode/4194304   4.5029 ms    888.32 MiB/s
```