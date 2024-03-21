use anyhow::Result;
use loadtimer::{
    cli::Cli,
    dump::{clear_n_lines, print_proc_metrics},
    eval::ProcMetrics,
    get_user_hz,
    sample::Sampler,
};
use std::time::Duration;

fn main() -> Result<()> {
    let args: Cli = argh::from_env();
    let user_hz = get_user_hz()? as f64;

    println!("Measuring CPU usage of PIDs {:?}", args.pids);
    println!("{} sample(s) of {}s", args.num_samples, args.sample_secs);

    if args.interactive {
        interactive(args, user_hz)
    } else {
        oneshot(args, user_hz)
    }
}

fn oneshot(args: Cli, user_hz: f64) -> Result<()> {
    let sample_duration = Duration::from_secs(args.sample_secs);
    let mut sampler = Sampler::new(&args.pids, args.num_samples, args.with_threads)?;

    for _ in 0..args.num_samples {
        sampler.sample(sample_duration)?;
    }

    let metrics = sampler
        .buffers()
        .map(|buf| ProcMetrics::from_buffer(buf, user_hz));
    let descriptions = sampler.buffers().map(|x| x.name.as_ref());

    println!();
    print_proc_metrics(metrics, descriptions);

    Ok(())
}

fn interactive(args: Cli, user_hz: f64) -> Result<()> {
    let sample_duration = Duration::from_secs(args.sample_secs);
    let mut sampler = Sampler::new(&args.pids, args.num_samples, args.with_threads)?;

    for _ in 0..args.num_samples {
        sampler.sample(sample_duration)?;
    }

    println!();

    loop {
        let metrics = sampler
            .buffers()
            .map(|buf| ProcMetrics::from_buffer(buf, user_hz));
        let descriptions = sampler.buffers().map(|x| x.name.as_ref());

        print_proc_metrics(metrics, descriptions);

        sampler.sample(sample_duration)?;
        clear_n_lines(args.pids.len() + 1);
    }
}
