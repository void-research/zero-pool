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
- **Zero per worker queues** - single global queue structure = perfect workload balancing
- **Zero external dependencies** - standard library only and stable rust

Using a result-via-parameters pattern means workers place results into caller provided memory, removing thread transport overhead. The single global queue structure ensures optimal load balancing without the complexity of work-stealing or load redistribution algorithms.

Because the library uses raw pointers, you must ensure parameter structs (including any pointers they contain) remain valid until task completion, and that your task functions are thread-safe.

This approach allows complete freedom to optimise multi-threaded workloads any way you want.

#### Notes
- TaskFuture is easily clonable, but `wait()`/`wait_timeout()` must be called from the thread that submitted the task. `is_complete()` is safe to call from any thread.
- Zero-Pool supports both explicitly creating new thread pools (`ZeroPool::new`, `ZeroPool::with_workers`) and using the global instance (`zero_pool::global_pool`).
- Task functions take a single parameter (e.g. `&MyTaskParams`).

## Benchmarks (AMD 5900X, Linux 7.1)
```rust
rayon_heavy_compute         ... bench:   4,802,609.15 ns/iter (+/- 594,894.53)
zeropool_heavy_compute      ... bench:   4,515,778.35 ns/iter (+/- 384,069.02)
rayon_individual_tasks      ... bench:     771,468.31 ns/iter (+/- 52,976.92)
zeropool_individual_tasks   ... bench:   1,043,626.32 ns/iter (+/- 91,674.50)
rayon_task_overhead         ... bench:      30,815.31 ns/iter (+/- 2,444.29)
zeropool_task_overhead      ... bench:      31,169.77 ns/iter (+/- 2,899.86)
```

## Example Usage

### Submitting a Single Task

```rust
use zero_pool::ZeroPool;

struct CalculationParams {
    iterations: usize,
    result: *mut u64,
}

fn calculate_task(params: &CalculationParams) {
    let mut sum = 0u64;
    for i in 0..params.iterations {
        sum += i as u64;
    }
    unsafe { *params.result = sum; }
}

let pool = ZeroPool::new();
let mut result = 0u64;
let task = CalculationParams { iterations: 1000, result: &mut result };

let future = pool.submit_task(calculate_task, &task);
future.wait();

println!("Result: {}", result);
```

### Submitting Uniform Batches

Submits multiple tasks of the same type to the thread pool.

```rust
use zero_pool::ZeroPool;

struct ComputeParams {
    work_amount: usize,
    result: *mut u64,
}

fn compute_task(params: &ComputeParams) {
    let mut sum = 0u64;
    for i in 0..params.work_amount {
        sum += i as u64;
    }
    unsafe { *params.result = sum; }
}

let pool = ZeroPool::new();
let mut results = vec![0u64; 100];

let tasks: Vec<_> = results.iter_mut().enumerate().map(|(i, result)| {
    ComputeParams { work_amount: 1000 + i * 10, result }
}).collect();

let future = pool.submit_batch(compute_task, &tasks);
future.wait();

println!("First result: {}", results[0]);
```

### Submitting Multiple Independent Tasks

You can submit individual tasks and uniform batches in parallel:

```rust
use zero_pool::ZeroPool;

// Define first task type
struct ComputeParams {
    work_amount: usize,
    result: *mut u64,
}

fn compute_task(params: &ComputeParams) {
    let mut sum = 0u64;
    for i in 0..params.work_amount {
        sum += i as u64;
    }
    unsafe { *params.result = sum; }
}

// Define second task type
struct MultiplyParams { x: u64, y: u64, result: *mut u64 }

fn multiply_task(params: &MultiplyParams) {
    unsafe { *params.result = params.x * params.y; }
}

let pool = ZeroPool::new();

// Individual task
let mut single_result = 0u64;
let single_task_params = ComputeParams { work_amount: 1000, result: &mut single_result };

// Uniform batch
let mut batch_results = vec![0u64; 50];
let batch_task_params: Vec<_> = batch_results.iter_mut().enumerate()
    .map(|(i, result)| ComputeParams { work_amount: 500 + i, result })
    .collect();

// Submit all batches
let future1 = pool.submit_task(compute_task, &single_task_params);
let future2 = pool.submit_batch(compute_task, &batch_task_params);

// Wait on them in any order; completion order is not guaranteed
future1.wait();
future2.wait(); 

println!("Single: {}", single_result);
println!("Batch completed: {} tasks", batch_results.len());
```

### Using the Global Pool

If you prefer to share a single pool across your entire application, call the global accessor. The pool is created on first use and lives for the duration of the process.

```rust
use zero_pool::global_pool;

struct ExampleParams {
    work: usize,
    result: *mut u64,
}

fn example_task(params: &ExampleParams) {
    let mut sum = 0u64;
    for i in 0..params.work {
        sum = sum.wrapping_add(i as u64);
    }
    unsafe { *params.result = sum; }
}

let pool = global_pool();
let mut result = 0u64;
let params = ExampleParams { work: 1_000, result: &mut result };

pool.submit_task(example_task, &params).wait();
```