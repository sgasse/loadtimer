use std::{
    fs,
    time::{Duration, Instant},
};

use anyhow::Result;

use clap::Parser;
use micromath::statistics::{Mean, StdDev};
use nix::unistd::SysconfVar;
use prettytable::{row, Table};

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// PID of process to monitor
    pid: usize,

    /// Sample duration in seconds
    #[arg(short, long, default_value_t = 10)]
    sample_secs: u64,

    /// Break seconds
    #[arg(short, long, default_value_t = 5)]
    break_secs: u64,

    /// Number of sample points
    #[arg(short, long, default_value_t = 6)]
    num_samples: usize,
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
    fn print(&self, description: &str) {
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

        table.add_row(row![
            r =>
            description,
            format!("{:.1}", self.cpu_usage),
            format!("{:.1}", self.total_mean),
            format!("{:.1}", self.user_mean),
            format!("{:.2}", self.user_stddev),
            format!("{:.1}", self.system_mean),
            format!("{:.2}", self.system_stddev)
        ]);

        table.printstd();
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let user_hz = get_user_hz()?;

    println!("Benchmarking PID {}", args.pid);
    println!(
        "{} samples of {}s with {}s break in between",
        args.num_samples, args.sample_secs, args.break_secs
    );

    let metrics = sample_process(&args, user_hz)?;
    let description = format!("{} samples, {}s", args.num_samples, args.sample_secs);

    println!("");
    metrics.print(&description);

    Ok(())
}

fn sample_process(args: &Cli, user_hz: i64) -> Result<ProcMetrics> {
    let mut utimes = Vec::with_capacity(args.num_samples);
    let mut stimes = Vec::with_capacity(args.num_samples);
    let mut sample_times = Vec::with_capacity(args.num_samples);

    let sample_duration = Duration::from_secs(args.sample_secs);
    let break_duration = Duration::from_secs(args.break_secs);

    for _ in 0..args.num_samples {
        let (utime, stime, sample_time) = sample_user_sys_time(args.pid, sample_duration)?;
        utimes.push(utime as f32);
        stimes.push(stime as f32);
        sample_times.push(sample_time);
        std::thread::sleep(break_duration);
    }

    let user_mean = utimes.iter().cloned().mean();
    let system_mean = stimes.iter().cloned().mean();
    let total_mean = user_mean + system_mean;

    let user_stddev = utimes.stddev();
    let system_stddev = stimes.stddev();

    let total_time = utimes.iter().sum::<f32>() + stimes.iter().sum::<f32>();
    let total_sample_seconds: f32 = sample_times.iter().map(|t| t.as_secs_f32()).sum();
    let cpu_usage = (total_time / (user_hz as f32 * total_sample_seconds)) * 100.0;

    Ok(ProcMetrics {
        cpu_usage,
        total_mean,
        user_mean,
        user_stddev,
        system_mean,
        system_stddev,
    })
}

fn sample_user_sys_time(pid: usize, sample_duration: Duration) -> Result<(u64, u64, Duration)> {
    let time = Instant::now();
    let (u1, s1) = get_user_sys_time(pid)?;

    std::thread::sleep(sample_duration);

    let elapsed = time.elapsed();
    let (u2, s2) = get_user_sys_time(pid)?;

    Ok((u2 - u1, s2 - s1, elapsed))
}

fn get_user_sys_time(pid: usize) -> Result<(u64, u64)> {
    let path = format!("/proc/{pid}/stat");
    let stat_data = fs::read_to_string(path)?;

    let mut parts = stat_data.split(' ').into_iter().skip(13);

    let utime = parts.next().map(|v| v.parse::<u64>().ok()).flatten();
    let stime = parts.next().map(|v| v.parse::<u64>().ok()).flatten();

    Ok((utime.unwrap(), stime.unwrap()))
}

fn get_user_hz() -> Result<i64> {
    nix::unistd::sysconf(SysconfVar::CLK_TCK)?.ok_or(anyhow::anyhow!(
        "Could not retreive USER_HZ / SC_CLK_TCK / jiffies"
    ))
}
