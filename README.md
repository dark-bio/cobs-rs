# Fast COBS encoder and decoder

[![](https://img.shields.io/crates/v/darkbio-cobs.svg)](https://crates.io/crates/darkbio-cobs)
[![](https://docs.rs/darkbio-cobs/badge.svg)](https://docs.rs/darkbio-cobs)
[![](https://github.com/dark-bio/cobs-rs/workflows/tests/badge.svg)](https://github.com/dark-bio/cobs-rs/actions/workflows/ci.yml)

This repository is a *fast* implementation of [Consistent Overhead Byte Stuffing (COBS)](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing). It doesn't do much, but it does it fast. Although there might be eventual fixups and feature expansions for streaming codecs, assume the library is "done".

## Performance

You can run the benchmarks to see the performance of the safe versions, unsafe versions and the currently most popular Rust `cobs` package (`v0.5.1`).

```
% cargo bench -- --quiet
```

The report was post-processed to make it denser, but you will see something along the lines of:

```
Benchmark Environment:
  OS:        Darwin 26.6.2
  Kernel:    25.6.0
  Arch:      aarch64
  CPU:       Apple M2 Max
  Cores:     12
  Memory:    34.21 GB / 64.00 GB
  Build:     release
  Rustc:     rustc 1.98.0 (88d9e12ae 2026-08-18)

encode/16                       4.7363 ns    3.1461 GiB/s
encode/256                      8.2581 ns    28.871 GiB/s
encode/4096                     186.88 ns    20.412 GiB/s
encode/65536                    2.8347 µs    21.531 GiB/s
encode/262144                   12.585 µs    19.399 GiB/s
encode/1048576                  91.854 µs    10.632 GiB/s
encode/4194304                  461.15 µs    8.4706 GiB/s

decode/16                       5.4793 ns    2.7195 GiB/s
decode/256                      16.776 ns    14.212 GiB/s
decode/4096                     153.48 ns    24.854 GiB/s
decode/65536                    2.8512 µs    21.407 GiB/s
decode/262144                   13.617 µs    17.929 GiB/s
decode/1048576                  54.376 µs    17.959 GiB/s
decode/4194304                  219.44 µs    17.801 GiB/s

encode_unsafe/16                4.7660 ns    3.1266 GiB/s
encode_unsafe/256               8.2389 ns    28.938 GiB/s
encode_unsafe/4096              147.34 ns    25.890 GiB/s
encode_unsafe/65536             3.0086 µs    20.287 GiB/s
encode_unsafe/262144            12.849 µs    19.001 GiB/s
encode_unsafe/1048576           81.756 µs    11.945 GiB/s
encode_unsafe/4194304           447.19 µs    8.7351 GiB/s

decode_unsafe/16                5.4603 ns    2.7290 GiB/s
decode_unsafe/256               11.390 ns    20.932 GiB/s
decode_unsafe/4096              149.28 ns    25.554 GiB/s
decode_unsafe/65536             3.0110 µs    20.271 GiB/s
decode_unsafe/262144            13.684 µs    17.841 GiB/s
decode_unsafe/1048576           54.053 µs    18.067 GiB/s
decode_unsafe/4194304           217.63 µs    17.949 GiB/s

decode_nonzero/16               3.2584 ns    4.5731 GiB/s
decode_nonzero/256              7.6394 ns    31.209 GiB/s
decode_nonzero/4096             110.92 ns    34.392 GiB/s
decode_nonzero/65536            2.3777 µs    25.669 GiB/s
decode_nonzero/262144           10.774 µs    22.660 GiB/s
decode_nonzero/1048576          43.067 µs    22.676 GiB/s
decode_nonzero/4194304          175.23 µs    22.292 GiB/s

decode_nonzero_unsafe/16        2.9844 ns    4.9930 GiB/s
decode_nonzero_unsafe/256       4.1155 ns    57.932 GiB/s
decode_nonzero_unsafe/4096      109.13 ns    34.954 GiB/s
decode_nonzero_unsafe/65536     2.3143 µs    26.372 GiB/s
decode_nonzero_unsafe/262144    10.699 µs    22.820 GiB/s
decode_nonzero_unsafe/1048576   43.898 µs    22.246 GiB/s
decode_nonzero_unsafe/4194304   173.02 µs    22.577 GiB/s

jamesmunns/encode/16            9.8637 ns    1.5107 GiB/s
jamesmunns/encode/256           162.23 ns    1.4696 GiB/s
jamesmunns/encode/4096          2.4714 µs    1.5435 GiB/s
jamesmunns/encode/65536         38.318 µs    1.5928 GiB/s
jamesmunns/encode/262144        154.02 µs    1.5851 GiB/s
jamesmunns/encode/1048576       617.43 µs    1.5817 GiB/s
jamesmunns/encode/4194304       2.4702 ms    1.5813 GiB/s

jamesmunns/decode/16            15.250 ns    1000.6 MiB/s
jamesmunns/decode/256           221.90 ns    1.0744 GiB/s
jamesmunns/decode/4096          3.5196 µs    1.0839 GiB/s
jamesmunns/decode/65536         56.295 µs    1.0842 GiB/s
jamesmunns/decode/262144        226.07 µs    1.0799 GiB/s
jamesmunns/decode/1048576       909.18 µs    1.0741 GiB/s
jamesmunns/decode/4194304       3.6190 ms    1.0794 GiB/s
```
