use crate::sample::Buffer;

/// Mean and standard deviation calculated from the same sample.
pub struct MeanWithStdDev {
    pub mean: f64,
    pub stddev: f64,
}

impl MeanWithStdDev {
    /// Calculate mean and standard deviation from an iterator.
    pub fn from_clonable_iter(items: impl Iterator<Item = f64> + Clone) -> Self {
        let (num_elements, sum) = items
            .clone()
            .fold((0, 0.0), |acc, x| (acc.0 + 1, acc.1 + x));
        let mean = sum / num_elements as f64;

        let stddev =
            f64::sqrt(items.map(|x| f64::powi(x - mean, 2)).sum::<f64>() / num_elements as f64);

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
        let user = MeanWithStdDev::from_clonable_iter(buf.buf.iter().map(|s| s.times.utime as f64));
        let system =
            MeanWithStdDev::from_clonable_iter(buf.buf.iter().map(|s| s.times.stime as f64));
        let total = MeanWithStdDev::from_clonable_iter(
            buf.buf
                .iter()
                .map(|s| (s.times.utime + s.times.stime) as f64),
        );

        let sampled_usage = buf.buf.iter().map(|s| {
            ((s.times.utime + s.times.stime) as f64 / (user_hz * s.duration.as_secs_f64())) * 100.
        });
        let cpu_usage = MeanWithStdDev::from_clonable_iter(sampled_usage);

        Self {
            pid: buf.pid,
            cpu_usage,
            total,
            user,
            system,
        }
    }
}
