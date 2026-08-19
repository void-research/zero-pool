# Miri Verification

**Last Verified:** Zero-Pool v0.8.4

This directory contains integration tests verified by **Miri** (Rust's MIR interpreter) to ensure the thread pool is free of data races, deadlocks, memory leaks, and undefined behavior.

## Usage

Run the following command with Nightly Rust:

```bash
MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-preemption-rate=0" cargo +nightly miri test
```

### Flags Explained
* **`-Zmiri-tree-borrows`**: Uses the Tree Borrows aliasing model, which correctly verifies the library's safe function pointer erasure pattern.
* **`-Zmiri-preemption-rate=0`**: Forces a context switch at every possible opportunity to maximize race condition detection.

## Verification Log (Default Stacked Borrows)
```text
MIRIFLAGS="-Zmiri-preemption-rate=0" cargo +nightly miri test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/2c44ded350ccaf43/out/zero_pool-2c44ded350ccaf43)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/24a153c4e02b4614/out/integration-24a153c4e02b4614)

running 8 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_wait_timeout ... ok
test test_worker_counts ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.03s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 22) ... ok
test src/lib.rs - (line 23) ... ok
test src/pool.rs - pool::ZeroPool::submit_task (line 69) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 40) ... ok
test src/pool.rs - pool::ZeroPool::submit_batch (line 99) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

all doctests ran in 0.31s; merged doctests compilation took 0.01s
```

## Verification Log (Tree Borrows)
```text
MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-preemption-rate=0" cargo +nightly miri test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/2c44ded350ccaf43/out/zero_pool-2c44ded350ccaf43)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/24a153c4e02b4614/out/integration-24a153c4e02b4614)

running 8 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_wait_timeout ... ok
test test_worker_counts ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.03s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 22) ... ok
test src/pool.rs - pool::ZeroPool::submit_task (line 69) ... ok
test src/lib.rs - (line 23) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 40) ... ok
test src/pool.rs - pool::ZeroPool::submit_batch (line 99) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

all doctests ran in 0.58s; merged doctests compilation took 0.01s
```