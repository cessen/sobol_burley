# Sobol-Burley

[![Latest Release][crates-io-badge]][crates-io-url]
[![Documentation][docs-rs-img]][docs-rs-url]

A seedable Owen-scrambled Sobol sequence based on the paper [Practical Hash-based Owen Scrambling](http://www.jcgt.org/published/0009/04/01/) by Brent Burley, with an improved hash from [Building a Better LK Hash](https://psychopath.io/post/2021_01_30_building_a_better_lk_hash), and a larger set of direction vectors due to [Kuo et al](http://web.maths.unsw.edu.au/~fkuo/sobol/).

This crate is geared towards use in practical graphics applications, and as such has some limitations:

* The maximum sequence length is 2^16.
* The maximum number of supported dimensions is 256 (but you can get around this easily--see the crate documentation for how).
* It produces 32-bit floats rather than higher-precision 64-bit floats.

These are all trade-offs made for better performance and smaller code size.

Expanding this crate to be suitable for more general applications is a goal for the future.  However, efficient execution for graphics applications will always be the top priority.


## Basic usage

Basic usage is pretty straightforward.  The first parameter of `sample()` is the index of the sample you want, and the second parameter is the index of the dimension you want.  The parameters are zero-indexed, and the output is in the interval [0, 1).

```rust
// Print the first sixteen dimensions of sample 1.
for d in 0..16 {
    let n = sample(0, d, 0);
    println!("{}", n);
}

// Print the first sixteen dimensions of sample 2.
for d in 0..16 {
    let n = sample(1, d, 0);
    println!("{}", n);
}
```

If all you want is a single standard Owen-scrambled Sobol sequence, then this is all you need.

For more advanced usage--including how to use the third parameter, how to get around the 256-dimension limit, and how to calculate multiple dimensions at once with SIMD--see the crate documentation.


## License

The main code in this project is licensed under either of

* MIT license (licenses/MIT.txt or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0, (licenses/APACHE-2.0.txt or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

The Sobol direction numbers under `direction_numbers/` and some of the code in `build.rs` (demarcated by comments) is adapted from work by Stephen Joe and Frances Y. Kuo, and is under the 3-clause BSD license.

See `licenses/JOE_KUO.txt` for details.


## Contributing

Contributions are absolutely welcome!  Please keep in mind that this crate aims to be:

* no-std and allocation-free.  PRs that use allocation, etc. are very likely to be rejected.
* As small as it reasonably can be, including transitive dependencies.  PRs that pull in dependencies--especially deep dependency trees--are likely to be rejected unless they really pull their weight.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you will be licensed as above (MIT/Apache dual-license), without any additional terms or conditions.


[crates-io-badge]: https://img.shields.io/crates/v/sobol_burley.svg
[crates-io-url]: https://crates.io/crates/sobol_burley
[docs-rs-img]: https://docs.rs/sobol_burley/badge.svg
[docs-rs-url]: https://docs.rs/sobol_burley
