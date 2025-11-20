# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2024-05-23

### Added
- Added `sha2` dependency to `programs/pda-vanity/Cargo.toml` for manual hashing optimization.

### Changed
- Optimized the off-chain search tool (`search.rs`) for significantly faster PDA generation.
    - Implemented a zero-allocation Base58 suffix check.
    - Implemented a "Hash-First" strategy to defer expensive Elliptic Curve checks (`is_on_curve`) until after suffix verification.
    - Increased search speed from ~330k seeds/sec to ~2.8M seeds/sec (~8.5x speedup).
- Updated `README.md` with performance results and technical details of the optimization.

