Tests available:
test
figure5
yield_spin_loop_true
yield_spin_loop_false
async_match_deadlock
find_deadlock_config
minimal_deadlock

Modes available:
Fuzz
    FUZZ_A (aborts if incorrect schedule given)
    FUZZ_RR (round robin completion)
    FUZZ_RA (random completion)
    FUZZ_W (wraparound, doesn't quite work yet)
    FUZZ_PCT (sillier version of pct fuzzer, randomly chooses depths)
    in prog: FUZZ_PCT_PREEMPT
Non-Fuzz:
    RANDOM
    PCT
    ROUND_ROBIN

If you want to run a fuzzed test (you can configure using whichever test/mode you would like):
cargo afl build --bin fuzz_target
env RUST_LOG=shuttle=info AFL_BENCH_UNTIL_CRASH=1 cargo afl fuzz -i in -o out target/debug/fuzz_target -- --test async_match_deadlock --mode FUZZ_W

If you want to run a test that does not require fuzzing:
cargo build -—bin non_fuzz_target
env RUST_LOG=shuttle=info cargo run --bin non_fuzz_target — --test find_deadlock_config --mode ROUND_ROBIN
