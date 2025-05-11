<!-- Keep a Changelog guide -> https://keepachangelog.com -->

# alloy-reth-provider changelog

## [Unreleased]
- Add implementation for `StateReader`
- Replace dummy `ExecutionOutcome` with `StateReader` in mempool example
- Reuse tokio runtime for `StateProvider` if already running

# [1.3.12] - 2025-04-18
- Update reth to 1.3.12

# [1.3.11] - 2025-04-17
- Update reth to 1.3.11

# [1.3.10] - 2025-04-16
- Update reth to 1.3.10
- Add Mempool example

# [1.3.9] - 2025-04-14
- Update reth to 1.3.9

## [1.3.8-v2] - 2025-03-10
- Reorganize files to match the structure of reth
- Add implementation for `CanonStateSubscriptions`

## [1.3.8] - 2025-03-09
- Initial release
- Update reth to 1.3.8