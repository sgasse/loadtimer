use anyhow::Result;
use loadtimer::{
    cli::Cli, dump::print_proc_metrics, eval::ProcMetrics, get_user_hz, sample::Sampler,
};
use std::time::Duration;

fn main() -> Result<()> {
    let args: Cli = argh::from_env();
    let user_hz = get_user_hz()? as f64;

    println!("Benchmarking PIDs {:?}", args.pids);
    println!(
        "{} sample(s) of {}s with {}s break in between",
        args.num_samples, args.sample_secs, args.break_secs
    );

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

    // let metrics = sample_processes(&args, user_hz)?;
    let descriptions = args
        .pids
        .iter()
        .map(|pid| format!("{pid}, {} samples, {}s", args.num_samples, args.sample_secs));

    println!();
    print_proc_metrics(&metrics, descriptions);

    Ok(())
}
