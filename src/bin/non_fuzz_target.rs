use std::fs::File;
use std::env::args;
use tracing::trace;


fn match_test(test_name: &str) -> fn() {
    match test_name {
        "test" => ::benches::test,
        "figure5" => ::benches::figure5,
        "yield_spin_loop_true" => ::benches::yield_spin_loop_true,
        "yield_spin_loop_false" => ::benches::yield_spin_loop_false,
        "async_match_deadlock" => ::benches::demo_async_match_deadlock::async_match_deadlock,
        "find_deadlock_config" => ::benches::demo_bounded_buffer::test_bounded_buffer_find_deadlock_configuration,
        "minimal_deadlock" => ::benches::demo_bounded_buffer::test_bounded_buffer_minimal_deadlock,
        _ => ::benches::test,
    }
}

fn run_mode(mode: &str, test: fn()) {
    use shuttle::check_random;
    use shuttle::check_pct;
    
    match mode {
        "PCT" => {
            tracing::info!("running pct");
            // obviously, depth matters here. figure this out.
            check_pct(test, 10000, 4);
        }
        "RANDOM" => {
            tracing::info!("running random");
            check_random(test, 10000);
        }
        _ => {
            // default will just be random
            tracing::info!("no mode specified");
            check_random(test, 10000);
        }
    }
}

fn main() {
    const OUTPUT_PATH: &str = "output.log";
    let file = File::options().write(true).create(true).open(OUTPUT_PATH).unwrap();
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(file)
        .init();

    // have switch statement based off of flag?
    // fourth element in args will be the test we are running
    let args: Vec<String> = args().collect();
    println!("{args:?}");
    tracing::info!("{args:?}");
    
    if args.len() < 3{
        tracing::info!("args are {args:?}");
        tracing::info!("please specify a test case name\nfor options, check src/lib.rs\n");
        panic!("please specify a test case name\nfor options, check src/lib.rs\n");
    }
    if args.len() < 5{
        panic!("please specify a mode\noptions: FUZZ_W, FUZZ_RR, FUZZ_RA, FUZZ_A, PCT, RANDOM");
    }

    //  switching fuzzers not a thing, need to pass in mode programmatically

    let test = match_test(&args[3]);
    run_mode(&args[5], test);
    // shuttle::check_pct(::benches::test, 10000, 4);
}