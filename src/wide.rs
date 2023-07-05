//--------------------------------------------------------------------------
// x86/64 SSE
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
pub(crate) mod sse {
    use core::arch::x86_64::{
        __m128i, _mm_add_epi32, _mm_and_si128, _mm_or_si128, _mm_set1_epi32, _mm_set1_ps,
        _mm_set_epi32, _mm_setzero_si128, _mm_sll_epi32, _mm_slli_epi32, _mm_srl_epi32,
        _mm_srli_epi32, _mm_sub_epi32, _mm_sub_ps, _mm_xor_si128,
    };

    /// A packed set of four `u32`s.
    ///
    /// Addition, subtraction, and multiplication are all wrapping.
    ///
    /// Uses SIMD for computation on supported platforms.
    #[derive(Debug, Copy, Clone)]
    pub struct Int4 {
        v: __m128i,
    }

    impl Int4 {
        #[inline(always)]
        pub(crate) fn zero() -> Int4 {
            Int4 {
                v: unsafe { _mm_setzero_si128() },
            }
        }

        /// For testing.
        #[allow(dead_code)]
        fn get(self, i: usize) -> u32 {
            let n: [u32; 4] = unsafe { core::mem::transmute(self) };
            n[i]
        }

        /// Convert each integer to a float in [0.0, 1.0).
        ///
        /// Same behavior as
        /// [`parts::u32_to_f32_norm()`](`crate::parts::u32_to_f32_norm()`),
        /// applied to each integer individually.
        #[inline(always)]
        pub fn to_f32_norm(self) -> [f32; 4] {
            let n4 = unsafe {
                let a = _mm_srli_epi32(self.v, 9);
                let b = _mm_or_si128(a, _mm_set1_epi32(core::mem::transmute(0x3f800000u32)));
                _mm_sub_ps(core::mem::transmute(b), _mm_set1_ps(1.0))
            };

            unsafe { core::mem::transmute(n4) }
        }

        /// Reverse the order of the bits in each integer.
        ///
        /// Same behavior as `reverse_bits()` in the Rust standard
        /// library, applied to each integer individually.
        #[inline]
        pub fn reverse_bits(self) -> Int4 {
            let mut n = self.v;
            unsafe {
                // From http://aggregate.org/MAGIC/#Bit%20Reversal but SIMD
                // on four numbers at once.

                let y0 = _mm_set1_epi32(core::mem::transmute(0x55555555u32));
                n = _mm_or_si128(
                    _mm_and_si128(_mm_srli_epi32(n, 1), y0),
                    _mm_slli_epi32(_mm_and_si128(n, y0), 1),
                );

                let y1 = _mm_set1_epi32(core::mem::transmute(0x33333333u32));
                n = _mm_or_si128(
                    _mm_and_si128(_mm_srli_epi32(n, 2), y1),
                    _mm_slli_epi32(_mm_and_si128(n, y1), 2),
                );

                let y2 = _mm_set1_epi32(core::mem::transmute(0x0f0f0f0fu32));
                n = _mm_or_si128(
                    _mm_and_si128(_mm_srli_epi32(n, 4), y2),
                    _mm_slli_epi32(_mm_and_si128(n, y2), 4),
                );

                let y3 = _mm_set1_epi32(core::mem::transmute(0x00ff00ffu32));
                n = _mm_or_si128(
                    _mm_and_si128(_mm_srli_epi32(n, 8), y3),
                    _mm_slli_epi32(_mm_and_si128(n, y3), 8),
                );

                n = _mm_or_si128(_mm_srli_epi32(n, 16), _mm_slli_epi32(n, 16));

                Int4 { v: n }
            }
        }
    }

    impl core::ops::Mul for Int4 {
        type Output = Self;
        #[inline(always)]
        fn mul(self, other: Self) -> Int4 {
            // This only works with SSE 4.1 support.
            #[cfg(target_feature = "sse4.1")]
            unsafe {
                use core::arch::x86_64::_mm_mullo_epi32;
                Int4 {
                    v: _mm_mullo_epi32(self.v, other.v),
                }
            }

            // This works on all x86-64 chips.
            #[cfg(not(target_feature = "sse4.1"))]
            unsafe {
                use core::arch::x86_64::{_mm_mul_epu32, _mm_shuffle_epi32};
                let a = _mm_and_si128(
                    _mm_mul_epu32(self.v, other.v),
                    _mm_set_epi32(0, 0xffffffffu32 as i32, 0, 0xffffffffu32 as i32),
                );
                let b = _mm_and_si128(
                    _mm_mul_epu32(
                        _mm_shuffle_epi32(self.v, 0b11_11_01_01),
                        _mm_shuffle_epi32(other.v, 0b11_11_01_01),
                    ),
                    _mm_set_epi32(0, 0xffffffffu32 as i32, 0, 0xffffffffu32 as i32),
                );
                Int4 {
                    v: _mm_or_si128(a, _mm_shuffle_epi32(b, 0b10_11_00_01)),
                }
            }
        }
    }

    impl core::ops::MulAssign for Int4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: Self) {
            *self = *self * other;
        }
    }

    impl core::ops::Add for Int4 {
        type Output = Self;
        #[inline(always)]
        fn add(self, other: Self) -> Self {
            Int4 {
                v: unsafe { _mm_add_epi32(self.v, other.v) },
            }
        }
    }

    impl core::ops::AddAssign for Int4 {
        #[inline(always)]
        fn add_assign(&mut self, other: Self) {
            *self = *self + other;
        }
    }

    impl core::ops::Sub for Int4 {
        type Output = Self;
        #[inline(always)]
        fn sub(self, other: Self) -> Self {
            Int4 {
                v: unsafe { _mm_sub_epi32(self.v, other.v) },
            }
        }
    }

    impl core::ops::SubAssign for Int4 {
        #[inline(always)]
        fn sub_assign(&mut self, other: Self) {
            *self = *self - other;
        }
    }

    impl core::ops::BitAnd for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, other: Self) -> Int4 {
            Int4 {
                v: unsafe { _mm_and_si128(self.v, other.v) },
            }
        }
    }

    impl core::ops::BitAndAssign for Int4 {
        #[inline(always)]
        fn bitand_assign(&mut self, other: Self) {
            *self = *self & other;
        }
    }

    impl core::ops::BitOr for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, other: Self) -> Int4 {
            Int4 {
                v: unsafe { _mm_or_si128(self.v, other.v) },
            }
        }
    }

    impl core::ops::BitOrAssign for Int4 {
        #[inline(always)]
        fn bitor_assign(&mut self, other: Self) {
            *self = *self | other;
        }
    }

    impl core::ops::BitXor for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitxor(self, other: Self) -> Int4 {
            Int4 {
                v: unsafe { _mm_xor_si128(self.v, other.v) },
            }
        }
    }

    impl core::ops::BitXorAssign for Int4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, other: Self) {
            *self = *self ^ other;
        }
    }

    impl core::ops::Shl<i32> for Int4 {
        type Output = Self;
        #[inline(always)]
        fn shl(self, other: i32) -> Int4 {
            Int4 {
                v: unsafe { _mm_sll_epi32(self.v, _mm_set_epi32(0, 0, 0, other)) },
            }
        }
    }

    impl core::ops::Shr<i32> for Int4 {
        type Output = Self;
        #[inline(always)]
        fn shr(self, other: i32) -> Int4 {
            Int4 {
                v: unsafe { _mm_srl_epi32(self.v, _mm_set_epi32(0, 0, 0, other)) },
            }
        }
    }

    impl From<[u32; 4]> for Int4 {
        #[inline(always)]
        fn from(v: [u32; 4]) -> Self {
            Int4 {
                v: unsafe { core::mem::transmute(v) },
            }
        }
    }

    impl From<Int4> for [u32; 4] {
        #[inline(always)]
        fn from(i: Int4) -> [u32; 4] {
            unsafe { core::mem::transmute(i.v) }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_array() {
            let a = Int4::from([1, 2, 3, 4]);
            assert_eq!(a.get(0), 1);
            assert_eq!(a.get(1), 2);
            assert_eq!(a.get(2), 3);
            assert_eq!(a.get(3), 4);
        }

        #[test]
        fn shr() {
            let a = Int4::from([0xffffffff; 4]) >> 16;
            assert_eq!(a.get(0), 0x0000ffff);
            assert_eq!(a.get(1), 0x0000ffff);
            assert_eq!(a.get(2), 0x0000ffff);
            assert_eq!(a.get(3), 0x0000ffff);
        }

        #[test]
        fn shl() {
            let a = Int4::from([0xffffffff; 4]) << 16;
            assert_eq!(a.get(0), 0xffff0000);
            assert_eq!(a.get(1), 0xffff0000);
            assert_eq!(a.get(2), 0xffff0000);
            assert_eq!(a.get(3), 0xffff0000);
        }

        #[test]
        fn to_f32_norm() {
            let a = Int4::from([0x00000000; 4]);
            let b = Int4::from([0x80000000; 4]);
            let c = Int4::from([0xffffffff; 4]);

            let a2 = a.to_f32_norm();
            let b2 = b.to_f32_norm();
            let c2 = c.to_f32_norm();

            assert_eq!(a2, [0.0, 0.0, 0.0, 0.0]);
            assert_eq!(b2, [0.5, 0.5, 0.5, 0.5]);
            assert!(c2[0] > 0.99999 && c2[0] < 1.0);
            assert!(c2[1] > 0.99999 && c2[1] < 1.0);
            assert!(c2[2] > 0.99999 && c2[2] < 1.0);
            assert!(c2[3] > 0.99999 && c2[3] < 1.0);
        }

        #[test]
        fn reverse_bits() {
            let a = 0xcde7a64e_u32;
            let b = 0xdc69fbd9_u32;
            let c = 0x3238fec6_u32;
            let d = 0x1fb9ba8f_u32;

            assert_eq!(Int4::from([a; 4]).reverse_bits().get(0), a.reverse_bits());
            assert_eq!(Int4::from([b; 4]).reverse_bits().get(0), b.reverse_bits());
            assert_eq!(Int4::from([c; 4]).reverse_bits().get(0), c.reverse_bits());
            assert_eq!(Int4::from([d; 4]).reverse_bits().get(0), d.reverse_bits());
        }
    }
}
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
pub use sse::Int4;

//--------------------------------------------------------------------------
// Fallback
#[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
pub(crate) mod fallback {
    /// A packed set of four `u32`s.
    ///
    /// Uses SIMD for computation on supported platforms.
    #[derive(Debug, Copy, Clone)]
    #[repr(align(16))]
    pub struct Int4 {
        v: [u32; 4],
    }

    impl Int4 {
        #[inline(always)]
        pub(crate) fn zero() -> Int4 {
            Int4 { v: [0, 0, 0, 0] }
        }

        /// Convert each integer to a float in [0.0, 1.0).
        ///
        /// Same behavior as
        /// [`parts::u32_to_f32_norm()`](`crate::parts::u32_to_f32_norm()`),
        /// applied to each integer individually.
        #[inline(always)]
        pub fn to_f32_norm(self) -> [f32; 4] {
            [
                f32::from_bits((self.v[0] >> 9) | 0x3f800000) - 1.0,
                f32::from_bits((self.v[1] >> 9) | 0x3f800000) - 1.0,
                f32::from_bits((self.v[2] >> 9) | 0x3f800000) - 1.0,
                f32::from_bits((self.v[3] >> 9) | 0x3f800000) - 1.0,
            ]
        }

        /// Reverse the order of the bits in each integer.
        ///
        /// Same behavior as `reverse_bits()` in the Rust standard
        /// library, applied to each integer individually.
        #[inline(always)]
        pub fn reverse_bits(self) -> Int4 {
            Int4 {
                v: [
                    self.v[0].reverse_bits(),
                    self.v[1].reverse_bits(),
                    self.v[2].reverse_bits(),
                    self.v[3].reverse_bits(),
                ],
            }
        }
    }

    impl core::ops::Mul for Int4 {
        type Output = Self;
        #[inline(always)]
        fn mul(self, other: Self) -> Int4 {
            Int4 {
                v: [
                    self.v[0].wrapping_mul(other.v[0]),
                    self.v[1].wrapping_mul(other.v[1]),
                    self.v[2].wrapping_mul(other.v[2]),
                    self.v[3].wrapping_mul(other.v[3]),
                ],
            }
        }
    }

    impl core::ops::MulAssign for Int4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: Self) {
            *self = *self * other;
        }
    }

    impl core::ops::Add for Int4 {
        type Output = Self;
        #[inline(always)]
        fn add(self, other: Self) -> Self {
            Int4 {
                v: [
                    self.v[0].wrapping_add(other.v[0]),
                    self.v[1].wrapping_add(other.v[1]),
                    self.v[2].wrapping_add(other.v[2]),
                    self.v[3].wrapping_add(other.v[3]),
                ],
            }
        }
    }

    impl core::ops::AddAssign for Int4 {
        #[inline(always)]
        fn add_assign(&mut self, other: Self) {
            *self = *self + other;
        }
    }

    impl core::ops::Sub for Int4 {
        type Output = Self;
        #[inline(always)]
        fn sub(self, other: Self) -> Self {
            Int4 {
                v: [
                    self.v[0].wrapping_sub(other.v[0]),
                    self.v[1].wrapping_sub(other.v[1]),
                    self.v[2].wrapping_sub(other.v[2]),
                    self.v[3].wrapping_sub(other.v[3]),
                ],
            }
        }
    }

    impl core::ops::SubAssign for Int4 {
        #[inline(always)]
        fn sub_assign(&mut self, other: Self) {
            *self = *self - other;
        }
    }

    impl core::ops::BitAnd for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, other: Self) -> Int4 {
            Int4 {
                v: [
                    self.v[0] & other.v[0],
                    self.v[1] & other.v[1],
                    self.v[2] & other.v[2],
                    self.v[3] & other.v[3],
                ],
            }
        }
    }

    impl core::ops::BitAndAssign for Int4 {
        #[inline(always)]
        fn bitand_assign(&mut self, other: Self) {
            *self = *self & other;
        }
    }

    impl core::ops::BitOr for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, other: Self) -> Int4 {
            Int4 {
                v: [
                    self.v[0] | other.v[0],
                    self.v[1] | other.v[1],
                    self.v[2] | other.v[2],
                    self.v[3] | other.v[3],
                ],
            }
        }
    }

    impl core::ops::BitOrAssign for Int4 {
        #[inline(always)]
        fn bitor_assign(&mut self, other: Self) {
            *self = *self | other;
        }
    }

    impl core::ops::BitXor for Int4 {
        type Output = Self;
        #[inline(always)]
        fn bitxor(self, other: Self) -> Int4 {
            Int4 {
                v: [
                    self.v[0] ^ other.v[0],
                    self.v[1] ^ other.v[1],
                    self.v[2] ^ other.v[2],
                    self.v[3] ^ other.v[3],
                ],
            }
        }
    }

    impl core::ops::BitXorAssign for Int4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, other: Self) {
            *self = *self ^ other;
        }
    }

    impl core::ops::Shl<i32> for Int4 {
        type Output = Self;
        #[inline(always)]
        fn shl(self, other: i32) -> Int4 {
            Int4 {
                v: [
                    self.v[0] << other,
                    self.v[1] << other,
                    self.v[2] << other,
                    self.v[3] << other,
                ],
            }
        }
    }

    impl core::ops::Shr<i32> for Int4 {
        type Output = Self;
        #[inline(always)]
        fn shr(self, other: i32) -> Int4 {
            Int4 {
                v: [
                    self.v[0] >> other,
                    self.v[1] >> other,
                    self.v[2] >> other,
                    self.v[3] >> other,
                ],
            }
        }
    }

    impl From<[u32; 4]> for Int4 {
        #[inline(always)]
        fn from(v: [u32; 4]) -> Self {
            Int4 { v }
        }
    }

    impl From<Int4> for [u32; 4] {
        #[inline(always)]
        fn from(i: Int4) -> [u32; 4] {
            i.v
        }
    }
}
#[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
pub use fallback::Int4;
