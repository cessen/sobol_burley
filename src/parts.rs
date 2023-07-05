//! The building blocks for making a Sobol sampler.
//!
//! This module contains the internal components of the main samplers
//! in this crate.  You can use these to build alternative Sobol
//! samplers.  However, it is easy to mess things up in subtle ways.
//! So unless you have unique requirements it is recommended to stick
//! with the main samplers in the crate root.
//!
//! **Note:** many of the functions in this module return reversed-bit
//! integers, and take some of their parameters as reversed-bit
//! integers as well.  This is always indicated by a `_rev` postfix
//! in the function name (for return values) and parameter names
//! (for parameter values).
//!
//! # Examples
//!
//! A simple, non-scrambled Sobol sequence function:
//!
//! ```rust
//! # use sobol_burley::parts::{sobol_rev, u32_to_f32_norm};
//! fn sobol(i: u32, dimension: u32) -> f32 {
//!     let sobol_int = sobol_rev(i.reverse_bits(), dimension).reverse_bits();
//!
//!     u32_to_f32_norm(sobol_int)
//! }
//! ```
//!
//! A basic Owen-scrambled Sobol sequence function:
//!
//! ```rust
//! # use sobol_burley::parts::{sobol_rev, u32_to_f32_norm, owen_scramble_rev, hash};
//! fn sobol_owen(i: u32, dimension: u32) -> f32 {
//!     let sobol_int_rev = sobol_rev(i.reverse_bits(), dimension);
//!
//!     let sobol_owen_int = owen_scramble_rev(
//!         sobol_int_rev,
//!         hash(dimension),
//!     ).reverse_bits();
//!
//!     u32_to_f32_norm(sobol_owen_int)
//! }
//! ```

pub use crate::wide::Int4;

use crate::{NUM_DIMENSIONS, NUM_DIMENSION_SETS_4D, REV_VECTORS};

/// Compute one dimension of a single sample in the Sobol sequence.
#[inline]
pub fn sobol_rev(sample_index_rev: u32, dimension: u32) -> u32 {
    assert!(dimension < NUM_DIMENSIONS);

    // The direction vectors are organized for SIMD, so we
    // need to access them this way.
    let dimension_set = (dimension >> 2) as usize;
    let sub_dimension = (dimension & 0b11) as usize;

    // Compute the Sobol sample with reversed bits.
    let vecs = &REV_VECTORS[dimension_set];
    let mut sobol = 0u32;
    let mut index = sample_index_rev & 0xffff0000; // Only use the top 16 bits.
    let mut i = 0;
    while index != 0 {
        let j = index.leading_zeros();
        // Note: using `get_unchecked()` here instead gives about a 3%
        // performance boost.  I'm opting to leave that on the table for now,
        // for the sake of keeping the main code entirely safe.
        sobol ^= vecs[(i + j) as usize][sub_dimension];
        i += j + 1;
        index <<= j;
        index <<= 1;
    }

    sobol
}

/// Same as [`sobol_rev()`] except returns four dimensions at once.
///
/// **Note:** `dimension_set` indexes into sets of four dimensions:
///
/// * `0` -> `[dim0, dim1, dim2, dim3]`
/// * `1` -> `[dim4, dim5, dim6, dim7]`
/// * etc.
#[inline]
pub fn sobol_int4_rev(sample_index_rev: u32, dimension_set: u32) -> Int4 {
    assert!(dimension_set < NUM_DIMENSION_SETS_4D);

    // Compute the Sobol sample with reversed bits.
    let vecs = &REV_VECTORS[dimension_set as usize];
    let mut sobol = Int4::zero();
    let mut index = sample_index_rev & 0xffff0000; // Only use the top 16 bits.
    let mut i = 0;
    while index != 0 {
        let j = index.leading_zeros();
        // Note: using `get_unchecked()` here instead gives about a 3%
        // performance boost.  I'm opting to leave that on the table for now,
        // for the sake of keeping the main code entirely safe.
        sobol ^= vecs[(i + j) as usize].into();
        i += j + 1;
        index <<= j;
        index <<= 1;
    }

    sobol
}

/// Scramble `n` using a hash function that closely approximates a
/// reverse-bit Owen scramble.
///
/// Passing a different random `scramble` parameter results in a different
/// random Owen scramble.
///
/// Uses the hash function from
/// <https://psychopath.io/post/2021_01_30_building_a_better_lk_hash>
///
/// **IMPORTANT:** `scramble` must already be well randomized!  For
/// example, incrementing integers will not work.  In general, you should
/// either:
///
/// * Get `scramble` from a random source, or
/// * First pass `scramble` through a hash function like [`hash_u32()`]
///   to randomize it before passing it to this function.
#[inline(always)]
pub fn owen_scramble_rev(mut n_rev: u32, scramble: u32) -> u32 {
    n_rev ^= n_rev.wrapping_mul(0x3d20adea);
    n_rev = n_rev.wrapping_add(scramble);
    n_rev = n_rev.wrapping_mul((scramble >> 16) | 1);
    n_rev ^= n_rev.wrapping_mul(0x05526c56);
    n_rev ^= n_rev.wrapping_mul(0x53a22864);

    n_rev
}

/// Same as [`owen_scramble_rev()`], except on 4 integers at a time.
///
/// You can (and probably should) put a different random scramble value
/// in each lane of `scramble` to scramble each lane differently.
#[inline(always)]
pub fn owen_scramble_int4_rev(mut n_rev: Int4, scramble: Int4) -> Int4 {
    n_rev ^= n_rev * [0x3d20adea; 4].into();
    n_rev += scramble;
    n_rev *= (scramble >> 16) | [1; 4].into();
    n_rev ^= n_rev * [0x05526c56; 4].into();
    n_rev ^= n_rev * [0x53a22864; 4].into();

    n_rev
}

/// A fast 32-bit hash function.
///
/// From <https://github.com/skeeto/hash-prospector>
#[inline(always)]
pub fn hash(mut n: u32) -> u32 {
    n ^= 0xe6fe3beb; // So zero doesn't map to zero.

    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;

    n
}

/// Same as [`hash_u32()`] except on four numbers at once.
#[inline(always)]
pub fn hash_int4(mut n: Int4) -> Int4 {
    n ^= [0xe6fe3beb; 4].into(); // So zero doesn't map to zero.

    n ^= n >> 16;
    n *= [0x7feb352d; 4].into();
    n ^= n >> 15;
    n *= [0x846ca68b; 4].into();
    n ^= n >> 16;

    n
}

/// Convert a `u32` to a float in [0.0, 1.0).
///
/// This maps the full range of `u32` to the [0, 1) range.
#[inline(always)]
pub fn u32_to_f32_norm(n: u32) -> f32 {
    f32::from_bits((n >> 9) | 0x3f800000) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn to_norm_f32() {
        assert_eq!(u32_to_f32_norm(0), 0.0);
        assert!(u32_to_f32_norm(core::u32::MAX) < 1.0);
    }
}
