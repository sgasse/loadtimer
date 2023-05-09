use std::{fs, time::Duration};

use anyhow::Result;

use clap::Parser;
use micromath::statistics::{Mean, StdDev};
use prettytable::{row, Table};

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// PID of process to monitor
    pid: usize,

    /// Sample duration in seconds
    #[arg(short, long, default_value_t = 20)]
    sample_secs: u64,

    /// Break seconds
    #[arg(short, long, default_value_t = 10)]
    break_secs: u64,

    /// Number of sample points
    #[arg(short, long, default_value_t = 6)]
    num_samples: usize,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    println!("Benchmarking PID {}", args.pid);
    println!(
        "{} samples of {}s with {}s break in between",
        args.num_samples, args.sample_secs, args.break_secs
    );

    let mut utimes = Vec::with_capacity(args.num_samples);
    let mut stimes = Vec::with_capacity(args.num_samples);

    let sample_duration = Duration::from_secs(args.sample_secs);
    let break_duration = Duration::from_secs(args.break_secs);

    for _ in 0..args.num_samples {
        let (utime, stime) = sample_user_sys_time(args.pid, sample_duration)?;
        utimes.push(utime as f32);
        stimes.push(stime as f32);
        std::thread::sleep(break_duration);
    }

    let utimes_mean = utimes.iter().cloned().mean();
    let stimes_mean = stimes.iter().cloned().mean();

    let utimes_stddev = utimes.stddev();
    let stimes_stddev = stimes.stddev();

    let description = format!("{} samples, {}s", args.num_samples, args.sample_secs);

    print_times(
        &description,
        utimes_mean,
        utimes_stddev,
        stimes_mean,
        stimes_stddev,
    );

    Ok(())
}

fn sample_user_sys_time(pid: usize, sample_duration: Duration) -> Result<(u64, u64)> {
    let (u1, s1) = get_user_sys_time(pid)?;
    std::thread::sleep(sample_duration);
    let (u2, s2) = get_user_sys_time(pid)?;

    Ok((u2 - u1, s2 - s1))
}

fn get_user_sys_time(pid: usize) -> Result<(u64, u64)> {
    let path = format!("/proc/{pid}/stat");
    let stat_data = fs::read_to_string(path)?;

    let mut parts = stat_data.split(' ').into_iter().skip(13);

    let utime = parts.next().map(|v| v.parse::<u64>().ok()).flatten();
    let stime = parts.next().map(|v| v.parse::<u64>().ok()).flatten();

    Ok((utime.unwrap(), stime.unwrap()))
}

fn print_times(description: &str, umean: f32, ustddev: f32, smean: f32, sstddev: f32) {
    let mut table = Table::new();

    table.add_row(row![
        "Description",
        "utime mean",
        "utime stddev",
        "stime mean",
        "stime stddev"
    ]);

    table.add_row(row![
        r =>
        description,
        format!("{umean:.1}"),
        format!("{ustddev:.2}"),
        format!("{smean:.1}"),
        format!("{sstddev:.2}")
    ]);

    table.printstd();
}
