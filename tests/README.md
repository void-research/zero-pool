# Miri Verification

**Last Verified:** Zero-Pool v0.9.0

This directory contains integration tests verified by **Miri** (Rust's MIR interpreter) to ensure the thread pool is free of data races, deadlocks, memory leaks, and undefined behavior.

## Usage

Run the following command with Nightly Rust:

```bash
MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-preemption-rate=1" cargo +nightly miri test
```

### Flags Explained
* **`-Zmiri-tree-borrows`**: Uses the Tree Borrows aliasing model, which correctly verifies the library's safe function pointer erasure pattern.
* **`-Zmiri-preemption-rate=1`**: Forces a context switch at every possible opportunity to maximize race condition detection.

## Verification Log (Default Stacked Borrows)
```text
MIRIFLAGS="-Zmiri-preemption-rate=1" cargo +nightly miri test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/58f5a1e0ab8ada84/out/zero_pool-58f5a1e0ab8ada84)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/e694178bf727648b/out/integration-e694178bf727648b)

running 12 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_detached_batch_submission ... ok
test test_detached_submission ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_scope ... ok
test test_scope_panic_safety ... ok
test test_scope_wait_and_is_complete ... ok
test test_worker_counts ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.44s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 27) ... ok
test src/pool.rs - pool::ZeroPool::run (line 102) ... ok
test src/lib.rs - (line 8) ... ok
test src/pool.rs - pool::ZeroPool::scope (line 70) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 45) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

all doctests ran in 0.16s; merged doctests compilation took 0.01s
```

## Verification Log (Tree Borrows)
```text
MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-preemption-rate=1" cargo +nightly miri test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/58f5a1e0ab8ada84/out/zero_pool-58f5a1e0ab8ada84)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/e694178bf727648b/out/integration-e694178bf727648b)

running 12 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_detached_batch_submission ... ok
test test_detached_submission ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_scope ... ok
test test_scope_panic_safety ... ok
test test_scope_wait_and_is_complete ... ok
test test_worker_counts ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.39s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 27) ... ok
test src/pool.rs - pool::ZeroPool::run (line 102) ... ok
test src/lib.rs - (line 8) ... ok
test src/pool.rs - pool::ZeroPool::scope (line 70) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 45) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

all doctests ran in 0.18s; merged doctests compilation took 0.01s
```