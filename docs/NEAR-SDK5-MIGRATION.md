# NEAR SDK 5.x Migration Summary

## Key Fixes Applied

### 1. SDK 5.x Pattern Updates
- Migrated `collections` → `store` module
- Added `Balance` type alias for compatibility
- Fixed AccountId: `new_unchecked` → `parse()`
- Updated monetary values to `NearToken` type

### 2. Dependencies
- Added `borsh` to workspace with `derive` feature
- Added `resolver = "2"` to Cargo.toml
- Removed `near-sdk` from non-contract libraries

### 3. Contract Structure
- Consolidated all impl blocks into single `#[near]` block
- Fixed `FusionPlusContractExt` trait generation

### 4. Store Module API
- Updated `insert()` and `get()` signatures
- Used `get_mut()` to avoid UnorderedSet cloning
- Removed `len()` calls (not available in store)

### 5. Build Configuration
- Switched to `cargo-near` for all builds
- Fixed WASM path detection in scripts
- Both contracts now compile successfully

## Remaining Work
- Migrate UnorderedSet → IterableSet
- Resolve JsonSchema/AccountId compatibility for ABI
- Update tests for SDK 5.x patterns