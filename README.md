# Sobol-Burley

A seedable Owen-scrambled Sobol sequence based on the paper [Practical Hash-based Owen Scrambling](http://www.jcgt.org/published/0009/04/01/) by Brent Burley, with an improved hash from [Building a Better LK Hash](https://psychopath.io/post/2021_01_30_building_a_better_lk_hash), and a larger set of direction vectors due to [Kuo et al](http://web.maths.unsw.edu.au/~fkuo/sobol/).

This crate is geared towards use in practical graphics applications, and as such has some limitations:

* The maximum sequence length is 2^16.
* The maximum number of supported dimensions is 256 (but you can get around this easily--see the crate documentation for how).
* It produces 32-bit floats rather than higher-precision 64-bit floats.

These are all trade-offs made for better performance and smaller code size.

This crate also currently targets SIMD execution on the x86-64 architecture, which is why it always calculates 4 dimensions at a time.  It will still compile and run just fine on any architecture, but will be notably slower due to not utilizing SIMD.

Expanding this crate to be more general, both in application and target architectures, is a goal for the future.  However, efficient execution for graphics applications will always be the top priority.


## Basic usage

Basic usage is very simple.  The first parameter of `sample_4d()` is the index of the sample you want, and the second parameter is the index of the set (of four) dimensions you want.  Everything is zero-indexed.

```rust
// The first eight dimensions of sample 1.
let sample_1__d0_3 = sample_4d(0, 0, 0);  // Dimensions 1-4.
let sample_1__d4_7 = sample_4d(0, 1, 0);  // Dimensions 5-8.

// The first eight dimensions of sample 2.
let sample_2__d0_3 = sample_4d(1, 0, 0);  // Dimensions 1-4.
let sample_2__d4_7 = sample_4d(1, 1, 0);  // Dimensions 5-8.
```

If all you want is a standard Owen-scrambled Sobol sequence, then that's all you need.

For more advanced usage, including how to use the third parameter and how to get around the 256-dimension limit, see the crate documentation.


## License

### Main code:

Copyright (c) 2021 Nathan Vegdahl

This project is licensed under either of

* MIT license (licenses/MIT.txt or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0, (licenses/APACHE-2.0.txt or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

### Direction numbers and build.rs code:

The Sobol direction numbers under `direction_numbers/` and some of the code in `build.rs` (demarcated by comments) is adapted from work by Stephen Joe and Frances Y. Kuo, and is under the 3-clause BSD license.

See `licenses/JOE_KUO.txt` for details.


## Contributing

Contributions are absolutely welcome!  Please keep in mind that this crate aims to be:

* no-std and allocation-free.  PRs that use allocation, etc. are very likely to be rejected.
* As small as it reasonably can be, including transitive dependencies.  PRs that pull in dependencies--especially deep dependency trees--are likely to be rejected unless they really pull their weight.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you will be licensed as above (MIT/Apache dual-license), without any additional terms or conditions.
