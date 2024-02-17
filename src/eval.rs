use std::time::Duration;

use crate::sample::Buffer;

pub struct MeanWithStdDev {
    pub mean: f64,
    pub stddev: f64,
}

impl MeanWithStdDev {
    pub fn from_vec(items: Vec<f64>) -> Self {
        let n = items.len();

        let mean = if n != 0 {
            items.iter().sum::<f64>() / (n as f64)
        } else {
            f64::NAN
        };

        let stddev = if n != 0 {
            f64::sqrt(items.iter().map(|v| f64::powi(v - mean, 2)).sum::<f64>() / n as f64)
        } else {
            f64::NAN
        };

        Self { mean, stddev }
    }
}

pub struct ProcMetrics {
    pub pid: usize,
    pub cpu_usage: MeanWithStdDev,
    pub total: MeanWithStdDev,
    pub user: MeanWithStdDev,
    pub system: MeanWithStdDev,
}

impl ProcMetrics {
    pub fn from_buffer(buf: &Buffer, user_hz: f64) -> Self {
        let user = MeanWithStdDev::from_vec(buf.buf.iter().map(|s| s.0.utime as f64).collect());
        let system = MeanWithStdDev::from_vec(buf.buf.iter().map(|s| s.0.stime as f64).collect());
        let total = MeanWithStdDev::from_vec(
            buf.buf
                .iter()
                .map(|s| (s.0.utime + s.0.stime) as f64)
                .collect(),
        );

        let total_time: Duration = buf.buf.iter().map(|s| s.1).sum();

        let usage_factor = 100.0 / (user_hz * total_time.as_secs_f64());

        let cpu_usage = MeanWithStdDev {
            mean: total.mean * usage_factor,
            stddev: total.stddev * usage_factor,
        };

        Self {
            pid: buf.pid,
            cpu_usage,
            total,
            user,
            system,
        }
    }
}
