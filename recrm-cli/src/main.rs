use log::info;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    env_logger::init();

    let start = Instant::now();

    let input = env::args().nth(1).expect("No file specified");

    recrm_core::recursively_remove(PathBuf::from(input));

    let end = start.elapsed();
    info!(
        "elapsed: {}.{:03}",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );
}
