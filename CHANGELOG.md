# Changelog


## [Unreleased]


## [0.4.0] - 2022-07-16

- Expose sampler internals so people can build their own custom samplers if needed for specific situations.


## [0.3.1] - 2021-05-16

- Add a `simd` feature flag, to allow people to disable simd (and thus unsafe code).
- Documentation improvements.


## [0.3.0] - 2021-05-14

- Add support for computing just one dimension at a time.  This makes usage
  easier when performance isn't critical, and makes the documention a bit
  easier to follow.
- Panic in debug when the max sample count is exceeded.
- Reduce memory footprint, by storing only the half of the direction vector data that we actually use.
- Documentation improvements.


## [0.2.0] - 2021-05-11

- Renamed MAX_DIMENSION_SET to NUM_DIMENSION_SETS to better reflect its meaning.
- Some documentation improvements and cleanups.
- Very tiny performance improvements due to better u32-to-f32 conversion and
  leaner SSE bit reversal code.


## [0.1.0] - 2021-05-11

- First release.


[Unreleased]: https://github.com/cessen/sobol_burley/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/cessen/sobol_burley/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/cessen/sobol_burley/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/cessen/sobol_burley/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/cessen/sobol_burley/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cessen/sobol_burley/releases/tag/v0.1.0
