use shuttle::sync::*;
use shuttle::thread;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

const TEST_LENGTH: usize = 20;

// basic test
pub fn test() {
    let lock = Arc::new(Mutex::new(0));
    let lock_clone = lock.clone();

    let _thd = thread::spawn(move || {
        *lock.lock().unwrap() = 1;
    });

    let read = *lock_clone.lock().unwrap();
    // tracing::info!("read value {:?}", read);
    assert_eq!(read, 0);
}

/// Based on Fig 5 of the PCT paper. A randomized scheduler struggles here because it must choose
/// to continually schedule thread 1 until it terminates, which happens with chance 2^TEST_LENGTH.
/// On the other hand, this is a bug depth of 1, so PCT should find it with p = 1/2.
pub fn figure5() {
    let lock = Arc::new(Mutex::new(0usize));
    let lock_clone = Arc::clone(&lock);

    thread::spawn(move || {
        for _ in 0..TEST_LENGTH {
            thread::sleep(Duration::from_millis(1));
        }

        *lock_clone.lock().unwrap() = 1;
    });

    let l = lock.lock().unwrap();
    assert_ne!(*l, 1, "thread 1 ran to completion");
}

// Check that PCT correctly deprioritizes a yielding thread. If it wasn't, there would be some
// iteration of this test where the yielding thread has the highest priority and so the others
// never make progress.
fn yield_spin_loop(use_yield: bool) {
    const NUM_THREADS: usize = 4;

    // let scheduler = PctScheduler::new(1, 100);
    // let mut config = Config::new();
    // config.max_steps = MaxSteps::FailAfter(50);
    // let runner = Runner::new(scheduler, config);
    // runner.run(move || {
    let count = Arc::new(AtomicUsize::new(0usize));

    let _thds = (0..NUM_THREADS)
        .map(|_| {
            let count = count.clone();
            thread::spawn(move || {
                count.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();

    while count.load(Ordering::SeqCst) < NUM_THREADS {
        if use_yield {
            thread::yield_now();
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }
    // });
}

pub fn yield_spin_loop_true() {
    yield_spin_loop(true);
}

pub fn yield_spin_loop_false() {
    yield_spin_loop(false);
}

pub mod demo_async_match_deadlock {
    use shuttle::future as tokio;
    use shuttle::sync::{Arc, RwLock};
    use std::time::Duration;

    /// We don't have an equivalent of `tokio::time::sleep`, but yielding has basically the same effect
    async fn sleep(_duration: Duration) {
        shuttle::future::yield_now().await;
    }

    #[derive(Default)]
    struct State {
        value: u64,
    }

    impl State {
        fn foo(&self) -> bool {
            self.value > 0
        }

        fn bar(&self) -> u64 {
            self.value
        }

        fn update(&mut self) {
            self.value += 1;
        }
    }

    // #[tokio::main(worker_threads = 2)]
    async fn mn() {
        let state: Arc<RwLock<State>> = Default::default();

        tokio::spawn({
            let state = state.clone();
            async move {
                loop {
                    println!("updating...");
                    state.write().unwrap().update();
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        for _ in 0..10 {
            match state.read().unwrap().foo() {
                true => {
                    println!("it's true!");
                    sleep(Duration::from_millis(1)).await;
                    println!("bar = {}", state.read().unwrap().bar());
                }
                false => {
                    println!("it's false!");
                }
            }
        }
        println!("okay done");
    }

    // #[should_panic(expected = "tried to acquire a RwLock it already holds")]
    pub fn async_match_deadlock() {
        tokio::block_on(mn());
    }

}

pub mod demo_bounded_buffer {
    // For symmetry we clone some `Arc`s even though we could just move them
    #![allow(clippy::redundant_clone)]

    use shuttle::rand::Rng;
    use shuttle::rand::thread_rng;
    use shuttle::sync::{Condvar, Mutex};
    use shuttle::thread;
    use std::sync::Arc;

    /// This file implements the example from a blog post about Coyote (P#):
    ///   https://cloudblogs.microsoft.com/opensource/2020/07/14/extreme-programming-meets-systematic-testing-using-coyote/
    /// The comments in this file are quotes from that blog post.

    /// Let’s walk through how Coyote can easily solve the programming problem posed by Tom Cargill. He
    /// shared a BoundedBuffer implementation written in Java with a known, but tricky, deadlock bug.
    ///
    /// The BoundedBuffer implements a buffer of fixed length with concurrent writers adding items to
    /// the buffer and readers consuming items from the buffer. The readers wait if there are no items
    /// in the buffer and writers wait if the buffer is full, resuming only once a slot has been
    /// consumed by a reader. This is also known as a producer/consumer queue.
    ///
    /// The concrete ask was for the community to find a particular bug that Cargill knew about in the
    /// above program. The meta-ask was to come up with a methodology for catching such bugs rapidly.
    // Unlike in the C# and Java code, Rust doesn't have monitors. But a monitor is just a Mutex and a
    // Condvar anyway, so we implement it that way instead. Some of this code is not idiomatic Rust,
    // but is written this way to match the C# code more closely.
    #[derive(Clone)]
    struct BoundedBuffer<T: Copy> {
        inner: Arc<Mutex<Inner<T>>>,
        cond: Arc<Condvar>,
    }

    struct Inner<T: Copy> {
        buffer: Box<[T]>,
        buffer_size: usize,
        put_at: usize,
        take_at: usize,
        occupied: usize,
    }

    impl<T: Copy + Default> BoundedBuffer<T> {
        fn new(buffer_size: usize) -> Self {
            let inner = Inner {
                buffer: vec![T::default(); buffer_size].into_boxed_slice(),
                buffer_size,
                put_at: 0,
                take_at: 0,
                occupied: 0,
            };

            BoundedBuffer {
                inner: Arc::new(Mutex::new(inner)),
                cond: Arc::new(Condvar::new()),
            }
        }

        fn put(&self, x: T) {
            let mut this = self.inner.lock().unwrap();
            while this.occupied == this.buffer_size {
                this = self.cond.wait(this).unwrap();
            }

            this.occupied += 1;
            this.put_at %= this.buffer_size;
            let put_at = this.put_at;
            this.buffer[put_at] = x;
            this.put_at += 1;

            self.cond.notify_one();
        }

        fn take(&self) -> T {
            let mut this = self.inner.lock().unwrap();
            while this.occupied == 0 {
                this = self.cond.wait(this).unwrap();
            }

            this.occupied -= 1;
            this.take_at %= this.buffer_size;
            let result = this.buffer[this.take_at];
            this.take_at += 1;

            self.cond.notify_one();

            result
        }
    }

    fn reader(buffer: BoundedBuffer<usize>, iterations: usize) {
        for _ in 0..iterations {
            let _ = buffer.take();
        }
    }

    fn writer(buffer: BoundedBuffer<usize>, iterations: usize) {
        for i in 0..iterations {
            buffer.put(i);
        }
    }

/// The bug might only trigger in certain configurations, but not in all configurations. Can we use
/// Coyote to explore the state space of the configurations? Luckily, we can.
///
/// We can generate a random number of readers, writers, buffer length, and iterations, letting
/// Coyote explore these configurations. Coyote will also explore the Task interleavings in each
/// configuration. The following slightly more interesting Coyote test explores these
/// configurations, letting Coyote control the non-determinism introduced by these random variables
/// and the scheduling of the resulting number of tasks.
    // #[test]
    // #[should_panic(expected = "deadlock")]

    pub fn test_bounded_buffer_find_deadlock_configuration() {

        let mut rng = thread_rng();

        let buffer_size = rng.gen_range(0usize..5) + 1;
        let readers = rng.gen_range(0usize..5) + 1;
        let writers = rng.gen_range(0usize..5) + 1;
        let iterations = rng.gen_range(0usize..10) + 1;
        let total_iterations = iterations * readers;
        let writer_iterations = total_iterations / writers;
        let remainder = total_iterations % writers;

        tracing::info!(buffer_size, readers, writers, iterations);

        let buffer = BoundedBuffer::new(buffer_size);

        let mut tasks = vec![];

        for _ in 0..readers {
            let buffer = buffer.clone();
            tasks.push(thread::spawn(move || reader(buffer, iterations)));
        }

        for i in 0..writers {
            let buffer = buffer.clone();
            let mut w = writer_iterations;
            if i == writers - 1 {
                w += remainder;
            }
            tasks.push(thread::spawn(move || writer(buffer, w)));
        }

        for task in tasks {
            task.join().unwrap();
        }
    }

    /// Indeed, we now see clearly that there is a minimal test with two readers and one writer. We also
    /// see all these deadlocks can be found with a buffer size of one and a small number of iterations.
    #[allow(clippy::vec_init_then_push)]
    fn bounded_buffer_minimal() {
        let buffer = BoundedBuffer::new(1);

        let mut tasks = vec![];

        tasks.push({
            let buffer = buffer.clone();
            thread::spawn(move || reader(buffer, 5))
        });

        tasks.push({
            let buffer = buffer.clone();
            thread::spawn(move || reader(buffer, 5))
        });

        tasks.push({
            let buffer = buffer.clone();
            thread::spawn(move || writer(buffer, 10))
        });

        for task in tasks {
            task.join().unwrap();
        }
    }

    /// Now we can write the minimal test. We’ll use 10 iterations just to be sure it deadlocks often.
    // #[test]
    // #[should_panic(expected = "deadlock")]
    pub fn test_bounded_buffer_minimal_deadlock() {
        bounded_buffer_minimal()
    }
}