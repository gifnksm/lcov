# lcov-util

[![license](https://img.shields.io/crates/l/lcov-util.svg)](#license)
[![crates.io](https://img.shields.io/crates/v/lcov-util.svg)](https://crates.io/crates/lcov-util)
[![rust 1.56.1+ badge](https://img.shields.io/badge/rust-1.56.1+-93450a.svg)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![Rust CI](https://github.com/gifnksm/lcov/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/gifnksm/lcov/actions/workflows/rust-ci.yml)[![codecov](https://codecov.io/gh/gifnksm/lcov/branch/master/graph/badge.svg?token=uXsi5eu0RD)](https://codecov.io/gh/gifnksm/lcov)

Utility commands to manipulate and analyze LCOV tracefiles blazingly fast.

[LCOV] is a graphical front-end for coverage testing tool [gcov].
It collects gcov data for multiple source files and stores them into the file called as "tracefile".

The purpose of this crate is to operate the LCOV tracefile faster than [the original LCOV Perl
implementation][LCOV GitHub].

## Install

```console
cargo install lcov-util
```

## Performance

### Merge LCOV tracefiles

Comparing the execution of merging LCOV tracefiles, between 3 programs:

* `lcov 1.15`: [Latest released version of `LCOV`][lcov-release]
* `lcov master`: [Latest development version of `LCOV`][lcov-dev]
* `lcov-merge`: `lcov-merge` executable from [`lcov-util` v0.1.6][lcov-util]

with 3 datasets (generated by [`mkinfo` tool from LCOV repository][mkinfo]):

* small: merging 5 small tracefiles (2 tests, 5 source files)
* medium: merging 5 medium tracefiles (3 tests, 50 source files)
* large: merging 5 large tracefiles (2 tests, 500 source files)

|                  | small | medium |  large  |
| ---------------- | ----- | ------ | ------- |
| `lcov 1.15`      | 0.24s | 2.42s  | 22.18s  |
| `lcov master`    | 0.23s | 2.43s  | 22.11s  |
| **`lcov-merge`** | 0.01s | 0.20s  |  2.74s  |

In this benchmark, `lcov-merge` is 10-20x faster than `lcov 1.15` / `lcov-master`.

* Environment:
  * Arch Linux (5.10.16.3-microsoft-standard-WSL2)
  * AMD Ryzen 9 5950X

See [`benchsuite`](benchsuite) directory for details.

[lcov-release]: https://github.com/linux-test-project/lcov/releases/tag/v1.15
[lcov-dev]: https://github.com/linux-test-project/lcov/commit/d1d3024a8c82ee0a4c2afe948008a18415db9091
[lcov-util]: https://github.com/gifnksm/lcov/releases/tag/lcov-util_v0.1.6
[mkinfo]: https://github.com/linux-test-project/lcov/blob/d1d3024a8c82ee0a4c2afe948008a18415db9091/tests/bin/mkinfo

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.56.1**.
At least the last 3 versions of stable Rust are supported at any given time.

While a crate is pre-release status (0.x.x) it may have its MSRV bumped in a patch release.
Once a crate has reached 1.x, any MSRV bump will be accompanied with a new minor version.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in lcov by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[LCOV]: http://ltp.sourceforge.net/coverage/lcov.php
[gcov]: http://gcc.gnu.org/onlinedocs/gcc/Gcov.html
[LCOV GitHub]: https://github.com/linux-test-project/lcov
