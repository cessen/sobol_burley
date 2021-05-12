//! A seedable Owen-scrambled Sobol sequence.
//!
//! This is based on the paper [Practical Hash-based Owen
//! Scrambling](http://www.jcgt.org/published/0009/04/01/) by Brent Burley,
//! with an improved hash from [Building a Better LK
//! Hash](https://psychopath.io/post/2021_01_30_building_a_better_lk_hash),
//! and a larger set of direction vectors due to
//! [Kuo et al.](http://web.maths.unsw.edu.au/~fkuo/sobol/)
//!
//! This crate is geared towards use in practical graphics applications, and
//! as such has some limitations:
//!
//! * The maximum sequence length is 2^16.
//! * The maximum number of supported dimensions is 256.
//! * It produces 32-bit floats rather than higher-precision 64-bit floats.
//!
//!
//! ## Basic usage
//!
//! Basic usage is pretty straightforward.  The first parameter of
//! `sample_4d()` is the index of the sample you want, and the second
//! parameter is the index of the set (of four) dimensions you want.
//! The parameters are zero-indexed, and the outputs are in the
//! interval [0, 1).
//!
//! ```rust
//! # use sobol_burley::sample_4d;
//! // Print the first sixteen dimensions of sample 1.
//! for d in 0..4 {
//!     let [w, x, y, z] = sample_4d(0, d, 0);
//!     println!("{} {} {} {}", w, x, y, z);
//! }
//!
//! // Print the first sixteen dimensions of sample 2.
//! for d in 0..4 {
//!     let [w, x, y, z] = sample_4d(1, d, 0);
//!     println!("{} {} {} {}", w, x, y, z);
//! }
//! ```
//!
//! If all you want is a single standard Owen-scrambled Sobol sequence,
//! then this is all you need.  You can ignore the third parameter.
//!
//!
//! ## Advanced usage and seeding
//!
//! The third parameter of `sample_4d()` is a seed that produces statistically
//! independent Sobol sequences via the scrambling+shuffling technique in
//! Brent Burley's paper (linked above).
//!
//! One of the applications for this is to decorrelate the error between
//! related integral estimates.  For example, in a 3d renderer you might
//! pass a different seed to each pixel so that error in the pixel colors
//! shows up as noise instead of as structured artifacts.
//!
//! Another important application is "padding" the dimensions
//! of a Sobol sequence with another Sobol sequence.  For example, if you
//! need more than 256 dimensions you can do this:
//!
//! ```rust
//! use sobol_burley::{sample_4d, NUM_DIMENSION_SETS};
//!
//! // Generate 40000 dimensions.  (Remember the dimensions
//! // are generated in sets of four.)
//! for n in 0..10000 {
//!     let dimension_set_index = n % NUM_DIMENSION_SETS;
//!     let seed = n / NUM_DIMENSION_SETS;
//!
//!     let sample = sample_4d(0, dimension_set_index, seed);
//! }
//!```
//!
//! In this example, each contiguous set of 256 dimensions has a different
//! seed, and is therefore only randomly associated with the other sets even
//! though each set is stratified within itself.
//!
//! At first blush, being randomly associated might sound like a bad thing.
//! Being stratified is better, and is the whole point of using something like
//! the Sobol sequence.  However, at practical sample counts the
//! stratification of the Sobol sequence in high dimensions breaks down badly,
//! and randomness is often better.
//!
//! In fact, often using sets of just 2 to 4 stratified dimensions or so, and
//! mapping them carefully to your problem space, avoids artifacts and is a
//! win for convergence at practical sample counts.  See Burley's paper for
//! details.

#![no_std]
#![allow(clippy::unreadable_literal)]

mod wide;
use wide::Int4;

// This `include` provides `NUM_DIMENSIONS` and `REV_VECTORS`.
// See the build.rs file for how this included file is generated.
include!(concat!(env!("OUT_DIR"), "/vectors.inc"));

/// The number of available dimension sets.
pub const NUM_DIMENSION_SETS: u32 = NUM_DIMENSIONS / 4;

/// Compute four dimensions of a single sample in the Sobol sequence.
///
/// All numbers returned are in the interval [0, 1).
///
/// `sample_index` specifies which sample in the Sobol sequence to compute.
///
/// `dimension_set` specifies which four dimensions to compute. `0` yields the
/// first four dimensions, `1` the second four dimensions, and so on.
///
/// `seed` produces statistically independent Sobol sequences.  Passing two
/// different seeds will produce two different sequences that are only randomly
/// associated, with no stratification or correlation between them.
///
/// # Panics
///
/// Panics if `dimension_set` is greater than or equal to `NUM_DIMENSION_SETS`.
#[inline]
pub fn sample_4d(sample_index: u32, dimension_set: u32, seed: u32) -> [f32; 4] {
    assert!(dimension_set < NUM_DIMENSION_SETS);
    let vecs = &REV_VECTORS[dimension_set as usize];

    // Shuffle the index using the given seed to produce a unique statistically
    // independent Sobol sequence.
    let shuffled_rev_index = scramble(sample_index.reverse_bits(), seed);

    // Compute the Sobol sample with reversed bits.
    let mut sobol_rev = Int4::zero();
    let mut index = shuffled_rev_index & 0xffff0000; // Only use the top 16 bits.
    let mut i = 0;
    while index != 0 {
        let j = index.leading_zeros();
        // Note: using `get_unchecked()` here instead gives about a 3%
        // performance boost.  I'm opting to leave that on the table for now,
        // for the sake of keeping the main code entirely safe.
        sobol_rev ^= vecs[(i + j) as usize].into();
        i += j + 1;
        index <<= j;
        index <<= 1;
    }

    // Do Owen scrambling on the reversed-bits Sobol sample.
    // The multiply on `dimension_set` is to avoid accidental cancellation
    // with an incrementing or otherwise structured seed.
    let sobol_owen_rev = scramble_int4(sobol_rev, dimension_set.wrapping_mul(0x9c8f2d3b) ^ seed);

    // Un-reverse the bits and convert to floating point in [0, 1).
    sobol_owen_rev.reverse_bits().to_norm_floats()
}

//----------------------------------------------------------------------

/// Scrambles `n` using the hash function from
/// https://psychopath.io/post/2021_01_30_building_a_better_lk_hash
///
/// This is equivalent to Owen scrambling, but on reversed bits.
#[inline(always)]
fn scramble(mut n: u32, scramble: u32) -> u32 {
    let scramble = hash(scramble);

    n ^= n.wrapping_mul(0x3d20adea);
    n = n.wrapping_add(scramble);
    n = n.wrapping_mul((scramble >> 16) | 1);
    n ^= n.wrapping_mul(0x05526c56);
    n ^= n.wrapping_mul(0x53a22864);

    n
}

/// Same as `scramble()`, except does it on 4 integers at a time.
#[inline(always)]
fn scramble_int4(mut n: Int4, scramble: u32) -> Int4 {
    let scramble = hash_int4([scramble; 4].into());

    n ^= n * [0x3d20adea; 4].into();
    n += scramble;
    n *= (scramble >> 16) | [1; 4].into();
    n ^= n * [0x05526c56; 4].into();
    n ^= n * [0x53a22864; 4].into();

    n
}

/// A good 32-bit hash function.
/// From https://github.com/skeeto/hash-prospector
#[inline(always)]
fn hash(n: u32) -> u32 {
    let mut hash = n ^ 0x79c68e4a;

    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846ca68b);
    hash ^= hash >> 16;

    hash
}

/// Same as `hash()` except performs hashing on four numbers at once.
///
/// Each of the four numbers gets a different hash, so even if all input
/// numbers are the same, the outputs will still be different for each of them.
#[inline(always)]
fn hash_int4(n: Int4) -> Int4 {
    let mut hash = n ^ [0x912f69ba, 0x174f18ab, 0x691e72ca, 0xb40cc1b8].into();

    hash ^= hash >> 16;
    hash *= [0x7feb352d; 4].into();
    hash ^= hash >> 15;
    hash *= [0x846ca68b; 4].into();
    hash ^= hash >> 16;

    hash
}
