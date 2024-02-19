use crate::sample::Buffer;

/// Mean and standard deviation calculated from the same sample.
pub struct MeanWithStdDev {
    pub mean: f64,
    pub stddev: f64,
}

impl MeanWithStdDev {
    /// Calculate mean and standard deviation from a vector.
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
        let user = MeanWithStdDev::from_vec(buf.buf.iter().map(|s| s.times.utime as f64).collect());
        let system =
            MeanWithStdDev::from_vec(buf.buf.iter().map(|s| s.times.stime as f64).collect());
        let total = MeanWithStdDev::from_vec(
            buf.buf
                .iter()
                .map(|s| (s.times.utime + s.times.stime) as f64)
                .collect(),
        );

        let sampled_usage: Vec<_> = buf
            .buf
            .iter()
            .map(|s| {
                ((s.times.utime + s.times.stime) as f64 / (user_hz * s.duration.as_secs_f64()))
                    * 100.
            })
            .collect();
        let cpu_usage = MeanWithStdDev::from_vec(sampled_usage);

        Self {
            pid: buf.pid,
            cpu_usage,
            total,
            user,
            system,
        }
    }
}
