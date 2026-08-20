# Miri Verification

**Last Verified:** Zero-Pool v0.8.5

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
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/e806899093ca46ae/out/zero_pool-e806899093ca46ae)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/eee4e1d1ea3e6291/out/integration-eee4e1d1ea3e6291)

running 8 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_wait_timeout ... ok
test test_worker_counts ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.31s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 23) ... ok
test src/pool.rs - pool::ZeroPool::submit_task (line 70) ... ok
test src/lib.rs - (line 23) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 41) ... ok
test src/pool.rs - pool::ZeroPool::submit_batch (line 99) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

all doctests ran in 0.33s; merged doctests compilation took 0.01s
```

## Verification Log (Tree Borrows)
```text
MIRIFLAGS="-Zmiri-tree-borrows -Zmiri-preemption-rate=1" cargo +nightly miri test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/e806899093ca46ae/out/zero_pool-e806899093ca46ae)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/integration.rs (target/miri/x86_64-unknown-linux-gnu/debug/build/zero-pool/eee4e1d1ea3e6291/out/integration-eee4e1d1ea3e6291)

running 8 tests
test test_basic_functionality ... ok
test test_batch_submission ... ok
test test_consecutive_batches ... ok
test test_empty_batch_submission ... ok
test test_pool_lifecycle_and_recreation ... ok
test test_reclaim_trigger ... ok
test test_wait_timeout ... ok
test test_worker_counts ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.31s

   Doc-tests zero_pool

running 5 tests
test src/pool.rs - pool::ZeroPool::new (line 23) ... ok
test src/pool.rs - pool::ZeroPool::submit_task (line 70) ... ok
test src/lib.rs - (line 23) ... ok
test src/pool.rs - pool::ZeroPool::with_workers (line 41) ... ok
test src/pool.rs - pool::ZeroPool::submit_batch (line 99) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s

all doctests ran in 0.60s; merged doctests compilation took 0.01s
```