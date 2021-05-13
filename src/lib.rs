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
//! `sample()` is the index of the sample you want, and the second
//! parameter is the index of the dimension you want.  The parameters
//! are zero-indexed, and the output is in the interval [0, 1).
//!
//! ```rust
//! # use sobol_burley::sample;
//! // Print the first sixteen dimensions of sample 1.
//! for dimension in 0..16 {
//!     let n = sample(0, dimension, 0);
//!     println!("{}", n);
//! }
//!
//! // Print the first sixteen dimensions of sample 2.
//! for dimension in 0..16 {
//!     let n = sample(1, dimension, 0);
//!     println!("{}", n);
//! }
//! ```
//!
//! If all you want is a single standard Owen-scrambled Sobol sequence,
//! then this is all you need.  You can ignore the third parameter.
//!
//!
//! ## Seeding
//!
//! The third parameter of `sample()` is a seed that produces statistically
//! independent Sobol sequences via the shuffling technique in Brent Burley's
//! paper linked above.  (Note: different dimensions are already automatically
//! scrambled with different randomizations&mdash;you don't need the seed
//! parameter for that.)
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
//! # use sobol_burley::sample;
//! // Print 10000 dimensions of a single sample.
//! for dimension in 0..10000 {
//!     use sobol_burley::NUM_DIMENSIONS; // = 256
//!
//!     let dimension_index = dimension % NUM_DIMENSIONS;
//!     let seed = dimension / NUM_DIMENSIONS;
//!
//!     let n = sample(0, dimension_index, seed);
//!     println!("{}", n);
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
//! mapping them carefully to your problem space, can avoid artifacts and be a
//! win for convergence at practical sample counts.  See Burley's paper for
//! details.
//!
//!
//! # SIMD
//!
//! You can use `sample_4d()` to compute four dimensions of a sample at a time.
//! On the x86-64 architecture it will utilize SIMD for more efficient
//! computation, and is about 4x as fast.  On other architectures it will still
//! compute correct results, but SIMD won't be utilized.  (Expanding SIMD
//! support to more architectures is a future goal.)
//!
//! Importantly, `sample()` and `sample_4d()` always compute identical results:
//!
//! ```rust
//! # use sobol_burley::{sample, sample_4d};
//! for dimension_set in 0..10 {
//!     for i in 0..256 {
//!         let a = [
//!             sample(i, dimension_set * 4, 0),
//!             sample(i, dimension_set * 4 + 1, 0),
//!             sample(i, dimension_set * 4 + 2, 0),
//!             sample(i, dimension_set * 4 + 3, 0)
//!         ];
//!         let b = sample_4d(i, dimension_set, 0);
//!
//!         assert_eq!(a, b);
//!     }
//! }
//! ```
//!
//! The difference is only in performance and how the dimensions are specified.

#![no_std]
#![allow(clippy::unreadable_literal)]

mod wide;
use wide::Int4;

// This `include` provides `NUM_DIMENSIONS` and `REV_VECTORS`.
// See the build.rs file for how this included file is generated.
include!(concat!(env!("OUT_DIR"), "/vectors.inc"));

/// The number of available 4d dimension sets.
pub const NUM_DIMENSION_SETS_4D: u32 = NUM_DIMENSIONS / 4;

/// Compute one dimension of a single sample in the Sobol sequence.
///
/// All numbers returned are in the interval [0, 1).
///
/// `sample_index` specifies which sample in the Sobol sequence to compute.
///
/// `dimension` specifies which dimension to compute.
///
/// `seed` produces statistically independent Sobol sequences.  Passing two
/// different seeds will produce two different sequences that are only randomly
/// associated, with no stratification or correlation between them.
///
/// # Panics
///
/// Panics if `dimension` is greater than or equal to `NUM_DIMENSIONS`.
#[inline]
pub fn sample(sample_index: u32, dimension: u32, seed: u32) -> f32 {
    assert!(dimension < NUM_DIMENSIONS);

    // The direction vectors are organized for SIMD, so we
    // need to access them this way.
    let dimension_set = (dimension >> 2) as usize;
    let sub_dimension = (dimension & 0b11) as usize;

    // Shuffle the index using the given seed to produce a unique statistically
    // independent Sobol sequence.
    let shuffled_rev_index = scramble(sample_index.reverse_bits(), seed ^ 0x79c68e4a);

    // Compute the Sobol sample with reversed bits.
    let vecs = &REV_VECTORS[dimension_set];
    let mut sobol_rev = 0u32;
    let mut index = shuffled_rev_index & 0xffff0000; // Only use the top 16 bits.
    let mut i = 0;
    while index != 0 {
        let j = index.leading_zeros();
        // Note: using `get_unchecked()` here instead gives about a 3%
        // performance boost.  I'm opting to leave that on the table for now,
        // for the sake of keeping the main code entirely safe.
        sobol_rev ^= vecs[(i + j) as usize][sub_dimension];
        i += j + 1;
        index <<= j;
        index <<= 1;
    }

    // Do Owen scrambling on the reversed-bits Sobol sample.
    // The multiply on `seed` is to avoid accidental cancellation
    // with `dimension` on an incrementing or otherwise structured
    // seed.
    let sobol_owen_rev = scramble(
        sobol_rev,
        (dimension >> 2)
            ^ seed.wrapping_mul(0x9c8f2d3b)
            ^ [0x912f69ba, 0x174f18ab, 0x691e72ca, 0xb40cc1b8][sub_dimension],
    );

    // Un-reverse the bits and convert to floating point in [0, 1).
    f32::from_bits((sobol_owen_rev.reverse_bits() >> 9) | 0x3f800000) - 1.0
}

/// Compute four dimensions of a single sample in the Sobol sequence.
///
/// This is identical to `sample()`, but computes four dimensions at a time,
/// utilizing SIMD on x86-64 architectures.  Even without SIMD this can be
/// *slightly* faster, as there are some computations that get amortized over
/// the four dimensions.
///
/// `dimension_set` specifies which four dimensions to compute. `0` yields the
/// first four dimensions, `1` the second four dimensions, and so on.
///
/// # Panics
///
/// Panics if `dimension_set` is greater than or equal to `NUM_DIMENSION_SETS_4D`.
#[inline]
pub fn sample_4d(sample_index: u32, dimension_set: u32, seed: u32) -> [f32; 4] {
    assert!(dimension_set < NUM_DIMENSION_SETS_4D);

    // Shuffle the index using the given seed to produce a unique statistically
    // independent Sobol sequence.
    let shuffled_rev_index = scramble(sample_index.reverse_bits(), seed ^ 0x79c68e4a);

    // Compute the Sobol sample with reversed bits.
    let vecs = &REV_VECTORS[dimension_set as usize];
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
    // The multiply on `seed` is to avoid accidental cancellation
    // with `dimension_set` on an incrementing or otherwise structured
    // seed.
    let sobol_owen_rev = {
        let seed4: Int4 = [seed.wrapping_mul(0x9c8f2d3b); 4].into();
        let ds4: Int4 = [dimension_set; 4].into();
        scramble_int4(
            sobol_rev,
            ds4 ^ seed4 ^ [0x912f69ba, 0x174f18ab, 0x691e72ca, 0xb40cc1b8].into(),
        )
    };

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
fn scramble_int4(mut n: Int4, scramble: Int4) -> Int4 {
    let scramble = hash_int4(scramble);

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
fn hash(mut n: u32) -> u32 {
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;

    n
}

/// Same as `hash()` except performs hashing on four numbers at once.
#[inline(always)]
fn hash_int4(mut n: Int4) -> Int4 {
    n ^= n >> 16;
    n *= [0x7feb352d; 4].into();
    n ^= n >> 15;
    n *= [0x846ca68b; 4].into();
    n ^= n >> 16;

    n
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_1d_and_4d_match() {
        for s in 0..4 {
            for d in 0..8 {
                for n in 0..256 {
                    let a1 = sample(n, d * 4, s);
                    let b1 = sample(n, d * 4 + 1, s);
                    let c1 = sample(n, d * 4 + 2, s);
                    let d1 = sample(n, d * 4 + 3, s);

                    let [a2, b2, c2, d2] = sample_4d(n, d, s);

                    assert_eq!(a1, a2);
                    assert_eq!(b1, b2);
                    assert_eq!(c1, c2);
                    assert_eq!(d1, d2);
                }
            }
        }
    }
}
