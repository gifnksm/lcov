# lcov-util

[![Crates.io](https://img.shields.io/crates/v/lcov-util.svg)](https://crates.io/crates/lcov-util)
[![Travis CI Build Status](https://travis-ci.org/gifnksm/lcov.svg?branch=master)](https://travis-ci.org/gifnksm/lcov)
![LICENSE](https://img.shields.io/crates/l/lcov-util.svg)

Utility commands to operate and analyze LCOV trace file at blazingly fast.

[LCOV] is a graphical front-end for coverage testing tool [gcov].
It collects gcov data for multiple source files and stores them into the file called as "tracefile".

The purpose of this crate is to operate the LCOV tracefile faster than [the original LCOV Perl
implementation][LCOV GitHub].

## Install

```console
$ cargo install lcov-util
```

## Performance

The benchmark test is not performed yet.

## License

This project is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in lcov by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
