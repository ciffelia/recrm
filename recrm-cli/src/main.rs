use anyhow::Result;
use log::info;
use recrm_core::Task;
use recrm_core::{Event, OperationMode};
use std::env;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    env_logger::init();

    let input = env::args().nth(1).expect("No file specified");

    let start = Instant::now();

    let mut task = Task::new(&input)?;
    task.set_mode(OperationMode::ScanAndDelete)?;

    loop {
        let timeout = Duration::from_millis(500);
        let res = task.wait_for_event(timeout);

        let progress = task.get_progress();
        println!("{:?}", progress);

        match res {
            Some(Event::DeleteComplete) => {
                break;
            }
            None => {
                continue;
            }
        }
    }

    let end = start.elapsed();
    info!(
        "elapsed: {}.{:03}",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );

    Ok(())
}
