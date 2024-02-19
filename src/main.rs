use anyhow::Result;
use loadtimer::{
    cli::Cli, dump::print_proc_metrics, eval::ProcMetrics, get_process_command, get_user_hz,
    sample::Sampler,
};
use std::time::Duration;

fn main() -> Result<()> {
    let args: Cli = argh::from_env();
    let user_hz = get_user_hz()? as f64;

    println!("Benchmarking PIDs {:?}", args.pids);
    println!("{} sample(s) of {}s", args.num_samples, args.sample_secs);

    let sample_duration = Duration::from_secs(args.sample_secs);
    let mut sampler = Sampler::new(&args.pids, args.num_samples)?;

    for _ in 0..args.num_samples {
        sampler.sample(sample_duration)?;
    }

    let metrics: Vec<_> = sampler
        .buffers()
        .iter()
        .map(|buf| ProcMetrics::from_buffer(buf, user_hz))
        .collect();

    let descriptions = args
        .pids
        .iter()
        .map(|pid| get_process_command(*pid).unwrap_or_else(|_| format!("PID {pid}")));

    println!();
    print_proc_metrics(&metrics, descriptions);

    Ok(())
}
