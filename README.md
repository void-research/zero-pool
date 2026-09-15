# Zero-Pool: Consistent High-Performance Thread Pool

A FIFO MPMC thread pool with a single global queue and cooperative memory reclamation.

[Miri verification report](tests/README.md)

## Key Features:

- **Zero locks** - lock-free
- **Zero queue limit** - unbounded
- **Zero channels** - no std/crossbeam channel overhead
- **Zero virtual dispatch** - function pointer dispatch avoids vtable lookups
- **Zero core spinning** - all event-based
- **Zero result transport cost** - tasks write directly to caller-provided memory
- **Zero per-worker queues** - single global queue structure = perfect workload balancing
- **Zero external dependencies** - standard library only and stable Rust
- **Zero heap tracking overhead** - scoped tasks synchronize via stack-allocated counters (no `Arc` per task)

Using a result-via-parameters pattern means workers place results into caller-provided memory, removing thread transport overhead. The single global queue structure ensures optimal load balancing without the complexity of work-stealing or load redistribution algorithms.

#### Notes
- **`pool.run_detached`** allows fire-and-forget background tasks without waiting.
- Zero-Pool supports both explicitly creating new thread pools (`ZeroPool::new`, `ZeroPool::with_workers`) and using the global instance (`zero_pool::global_pool`).

## Example Usage

### Running Tasks

```rust
use zero_pool::ZeroPool;

struct Params { iterations: usize, result: *mut u64 }

fn compute(params: &Params) {
    let mut sum = 0;
    for i in 0..params.iterations { sum += i as u64; }
    unsafe { *params.result = sum; }
}

let pool = ZeroPool::new();

// Single task
let mut result = 0;
pool.run(compute, &[Params { iterations: 1000, result: &raw mut result }]);
println!("Result: {}", result);

// Batch
let mut results = vec![0; 100];
let tasks: Vec<_> = results.iter_mut().enumerate().map(|(i, r)| {
    Params { iterations: 1000 + i * 10, result: r }
}).collect();

pool.run(compute, &tasks);
println!("First result: {}", results[0]);
```

### Scoped Concurrent Tasks

Submit tasks of different types concurrently within a scope. All submitted tasks complete before the scope returns:

```rust
use zero_pool::ZeroPool;

struct ComputeParams { work_amount: usize, result: *mut u64 }
fn compute_task(params: &ComputeParams) {
    let mut sum = 0;
    for i in 0..params.work_amount { sum += i as u64; }
    unsafe { *params.result = sum; }
}

struct MultiplyParams { x: u64, y: u64, result: *mut u64 }
fn multiply_task(params: &MultiplyParams) {
    unsafe { *params.result = params.x * params.y; }
}

let pool = ZeroPool::new();
let mut compute_result = 0;
let mut multiply_result = 0;

let compute_params = [ComputeParams { work_amount: 1000, result: &raw mut compute_result }];
let multiply_params = [MultiplyParams { x: 6, y: 7, result: &raw mut multiply_result }];

pool.scope(|s| {
    // Both tasks are queued and execute in parallel
    s.run(compute_task, &compute_params);
    s.run(multiply_task, &multiply_params);
});

println!("Compute: {}, Multiply: {}", compute_result, multiply_result);
```

### Multi-Phase Coordination

Call `s.wait()` mid-scope to synchronize between computation phases:

```rust
let phase1_params = [Phase1Params { ... }];
let phase2_params = [Phase2Params { ... }];

pool.scope(|s| {
    s.run(phase1_task, &phase1_params);
    s.wait();
    // Phase 2 can use outputs from Phase 1
    s.run(phase2_task, &phase2_params);
});
```

### Using the Global Pool

```rust
use zero_pool::global_pool;

struct Params { work: usize, result: *mut u64 }
fn task(p: &Params) {
    let mut sum: u64 = 0;
    for i in 0..p.work { sum = sum.wrapping_add(i as u64); }
    unsafe { *p.result = sum; }
}

let mut result = 0;
global_pool().run(task, &[Params { work: 1_000, result: &raw mut result }]);
println!("Result: {}", result);
```