use std::{
    fs,
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};

use argh::FromArgs;
// use clap::Parser;
use micromath::statistics::{Mean, StdDev};
use nix::unistd::SysconfVar;
use prettytable::{row, Table};

// #[derive(Parser)]
// #[command(about, long_about = None)]
// struct Cli {
//     /// PID of process to monitor
//     pids: Vec<usize>,

//     /// Sample duration in seconds
//     #[arg(short, long, default_value_t = 30)]
//     sample_secs: u64,

//     /// Break seconds
//     #[arg(short, long, default_value_t = 0)]
//     break_secs: u64,

//     /// Number of sample points
//     #[arg(short, long, default_value_t = 2)]
//     num_samples: usize,
// }

#[derive(FromArgs)]
/// Measure CPU usage of processes.
struct Cli {
    /// pid of process to monitor
    #[argh(positional)]
    pids: Vec<usize>,

    /// sample duration in seconds
    #[argh(option, short = 's', default = "30")]
    sample_secs: u64,

    /// break seconds
    #[argh(option, short = 'b', default = "0")]
    break_secs: u64,

    /// number of sample points
    #[argh(option, short = 'n', default = "2")]
    num_samples: usize,
}

fn main() -> Result<()> {
    let args: Cli = argh::from_env();
    let user_hz = get_user_hz()?;

    println!("Benchmarking PIDs {:?}", args.pids);
    println!(
        "{} sample(s) of {}s with {}s break in between",
        args.num_samples, args.sample_secs, args.break_secs
    );

    let metrics = sample_processes(&args, user_hz)?;
    let descriptions = args
        .pids
        .iter()
        .map(|pid| format!("{pid}, {} samples, {}s", args.num_samples, args.sample_secs));

    println!("");
    print_proc_metrics(&metrics, descriptions);

    Ok(())
}

struct ProcMetrics {
    cpu_usage: f32,
    total_mean: f32,
    user_mean: f32,
    user_stddev: f32,
    system_mean: f32,
    system_stddev: f32,
}

impl ProcMetrics {
    fn from_utimes_stimes(
        utimes: &[f32],
        stimes: &[f32],
        sample_times: &[Duration],
        user_hz: i64,
    ) -> Self {
        let user_mean = utimes.iter().cloned().mean();
        let system_mean = stimes.iter().cloned().mean();
        let total_mean = user_mean + system_mean;

        let user_stddev = utimes.stddev();
        let system_stddev = stimes.stddev();

        let total_time = utimes.iter().sum::<f32>() + stimes.iter().sum::<f32>();
        let total_sample_seconds: f32 = sample_times.iter().map(|t| t.as_secs_f32()).sum();
        let cpu_usage = (total_time / (user_hz as f32 * total_sample_seconds)) * 100.0;

        ProcMetrics {
            cpu_usage,
            total_mean,
            user_mean,
            user_stddev,
            system_mean,
            system_stddev,
        }
    }
}

/// Collect several samples of user and system time per process and calculate mean, stddev and CPU usage.
fn sample_processes(args: &Cli, user_hz: i64) -> Result<Vec<ProcMetrics>> {
    if args.pids.is_empty() {
        bail!("no PIDs given");
    }

    // Setup buffers to store several samples of user and system time values per process.
    let mut utimes_per_pid_samples = vec![Vec::with_capacity(args.num_samples); args.pids.len()];
    let mut stimes_per_pid_samples = vec![Vec::with_capacity(args.num_samples); args.pids.len()];
    let mut sample_durations = Vec::with_capacity(args.num_samples);

    let sample_duration = Duration::from_secs(args.sample_secs);
    let break_duration = Duration::from_secs(args.break_secs);

    for _ in 0..args.num_samples {
        // Get user and system times per PID.
        let (utime_stime_vec, sample_time) = sample_user_sys_times(&args.pids, sample_duration)?;

        // Sort into vectors per PID.
        for (((utime, stime), utimes_per_pid), stimes_per_pid) in utime_stime_vec
            .into_iter()
            .zip(utimes_per_pid_samples.iter_mut())
            .zip(stimes_per_pid_samples.iter_mut())
        {
            utimes_per_pid.push(utime as f32);
            stimes_per_pid.push(stime as f32);
        }

        sample_durations.push(sample_time);

        // Wait specified time between samples.
        std::thread::sleep(break_duration);
    }

    // Turn user and system times into process metrics.
    Ok(utimes_per_pid_samples
        .into_iter()
        .zip(stimes_per_pid_samples.into_iter())
        .map(|(utimes, stimes)| {
            ProcMetrics::from_utimes_stimes(
                utimes.as_slice(),
                stimes.as_slice(),
                sample_durations.as_slice(),
                user_hz,
            )
        })
        .collect())
}

/// Sample user and system times for several processes in parallel.
fn sample_user_sys_times(
    pids: &[usize],
    sample_duration: Duration,
) -> Result<(Vec<(u64, u64)>, Duration)> {
    let time = Instant::now();

    // Get first value of user and system time per process.
    let mut utime_stime_vec = pids
        .iter()
        .map(|pid| get_user_sys_time(*pid))
        .collect::<Result<Vec<(u64, u64)>>>()?;

    std::thread::sleep(sample_duration);
    let elapsed = time.elapsed();

    // Update user and system times with the difference between the current and the previous value.
    for (pid, utime_stime) in pids.iter().zip(utime_stime_vec.iter_mut()) {
        let (u2, s2) = get_user_sys_time(*pid)?;

        utime_stime.0 = u2 - utime_stime.0;
        utime_stime.1 = s2 - utime_stime.1;
    }

    Ok((utime_stime_vec, elapsed))
}

/// Extract the user time and system time value from `/proc/{pid}/stat`.
fn get_user_sys_time(pid: usize) -> Result<(u64, u64)> {
    let path = format!("/proc/{pid}/stat");
    let stat_data = fs::read_to_string(path)?;

    let mut parts = stat_data.split(' ').into_iter().skip(13);

    let utime = parts
        .next()
        .map(|v| v.parse::<u64>().ok())
        .flatten()
        .context("failed to get utime")?;
    let stime = parts
        .next()
        .map(|v| v.parse::<u64>().ok())
        .flatten()
        .context("failed to get stime")?;

    Ok((utime, stime))
}

/// Get the `USER_HZ` constant, describing the number of jiffies.
///
/// The user and system time values reported in `/proc/{pid}/stat` no longer correspond to actual
/// cycles but to virtual cycles. The constant `USER_HZ` describes how many virtual cycles there
/// are in one second.
fn get_user_hz() -> Result<i64> {
    nix::unistd::sysconf(SysconfVar::CLK_TCK)?.ok_or(anyhow::anyhow!(
        "Could not retreive USER_HZ / SC_CLK_TCK / jiffies"
    ))
}

/// Print a table view of process metrics with their descriptions.
fn print_proc_metrics(proc_metrics: &[ProcMetrics], descriptions: impl Iterator<Item = String>) {
    let mut table = Table::new();

    use prettytable::format;

    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .padding(1, 1)
        .build();
    table.set_format(format);

    table.add_row(row![
        "Description",
        "CPU usage %",
        "total mean",
        "utime mean",
        "utime stddev",
        "stime mean",
        "stime stddev"
    ]);

    for (metric, description) in proc_metrics.iter().zip(descriptions) {
        table.add_row(row![
            r =>
            description,
            format!("{:.1}", metric.cpu_usage),
            format!("{:.1}", metric.total_mean),
            format!("{:.1}", metric.user_mean),
            format!("{:.2}", metric.user_stddev),
            format!("{:.1}", metric.system_mean),
            format!("{:.2}", metric.system_stddev)
        ]);
    }

    table.printstd();
}
