use std::fs::File;
use std::sync::Arc;

use shuttle::scheduler::Schedule;

fn main() {
    // const OUTPUT_PATH: &str = "output.log";
    // let file = File::options().write(true).create(true).open(OUTPUT_PATH).unwrap();
    // tracing_subscriber::fmt()
    //     .with_ansi(false)
    //     .with_writer(file)
    //     .init();

    // let scheduler = shuttle::scheduler::FuzzScheduler::new();
    // let runner = shuttle::Runner::new_fuzz(scheduler, Default::default());

    // let path = std::env::args().nth(1).expect("must provide a path to replay from");
    // let bytes = std::fs::read(path).expect("failed to read bytes");
    // let mut unstructured = arbitrary::Unstructured::new(&bytes);
    // let schedule: Schedule = arbitrary::Arbitrary::arbitrary(&mut unstructured).expect("invalid schedule");

    // println!("replaying crashing schedule: {:?}", schedule);

    // let f = Arc::new(::fuzztarget::test);
    // let mut i = 0;
    // runner.fuzz_inner(f, schedule, &mut i);
}