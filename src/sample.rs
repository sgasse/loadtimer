use anyhow::{Context, Result};
use std::{
    collections::VecDeque,
    fs,
    ops::Sub,
    time::{Duration, Instant},
};

pub struct ProcessTimes {
    pub utime: i64,
    pub stime: i64,
}

impl Sub for &ProcessTimes {
    type Output = ProcessTimes;

    fn sub(self, rhs: Self) -> Self::Output {
        ProcessTimes {
            utime: self.utime - rhs.utime,
            stime: self.stime - rhs.stime,
        }
    }
}

pub struct SamplePoint {
    pub times: ProcessTimes,
    pub duration: Duration,
}

pub struct Buffer {
    pub pid: usize,
    pub buf: VecDeque<SamplePoint>,
    pub last: ProcessTimes,
}

impl Buffer {
    pub fn new(pid: usize) -> Result<Self> {
        Self::with_capacity(pid, 0)
    }

    pub fn with_capacity(pid: usize, cap: usize) -> Result<Self> {
        let last = get_user_sys_time(pid)?;

        Ok(Self {
            pid,
            buf: VecDeque::with_capacity(cap),
            last,
        })
    }

    pub fn sample(&mut self, start: Instant) -> Result<()> {
        if self.buf.len() >= self.buf.capacity() {
            self.buf.pop_front();
        }

        let new_sample = get_user_sys_time(self.pid)?;

        self.buf.push_back(SamplePoint {
            times: &new_sample - &self.last,
            duration: start.elapsed(),
        });

        self.last = new_sample;

        Ok(())
    }
}

pub struct Sampler {
    start: Instant,
    buffers: Vec<Buffer>,
}

impl Sampler {
    pub fn new(pids: &[usize], num_samples: usize) -> Result<Self> {
        let start = Instant::now();

        let buffers = pids
            .iter()
            .map(|pid| Buffer::with_capacity(*pid, num_samples))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { start, buffers })
    }

    pub fn sample(&mut self, sample_duration: Duration) -> Result<()> {
        let sleep = sample_duration.saturating_sub(Instant::now() - self.start);

        std::thread::sleep(sleep);

        for buffer in self.buffers.iter_mut() {
            buffer.sample(self.start)?;
        }

        self.start = Instant::now();

        Ok(())
    }

    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }
}

/// Extract the user time and system time value from `/proc/{pid}/stat`.
fn get_user_sys_time(pid: usize) -> Result<ProcessTimes> {
    let path = format!("/proc/{pid}/stat");
    let stat_data = fs::read_to_string(path)?;

    let mut parts = stat_data.split(' ').skip(13);

    let utime = parts
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .context("failed to get utime")?;
    let stime = parts
        .next()
        .and_then(|v| v.parse::<i64>().ok())
        .context("failed to get stime")?;

    Ok(ProcessTimes { utime, stime })
}
