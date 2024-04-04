use anyhow::{Context, Result};
use std::{
    collections::{btree_map::Entry, BTreeMap, VecDeque},
    fs,
    ops::Sub,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::get_process_command;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SamplePoint {
    pub times: ProcessTimes,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct Buffer {
    pub path: PathBuf,
    pub name: String,
    pub buf: VecDeque<SamplePoint>,
    pub last: ProcessTimes,
}

impl Buffer {
    pub fn with_capacity(path: PathBuf, name: String, cap: usize) -> Result<Self> {
        let last = read_user_sys_time(&path)?;

        Ok(Self {
            path,
            name,
            buf: VecDeque::with_capacity(cap),
            last,
        })
    }

    pub fn from_pid(pid: usize, cap: usize) -> Result<Self> {
        let path: PathBuf = ["/proc", &pid.to_string(), "stat"].into_iter().collect();
        let name = get_process_command(pid).unwrap_or_else(|_| format!("PID {pid}"));
        Self::with_capacity(path, name, cap)
    }

    pub fn from_pid_tid(pid: usize, tid: usize, cap: usize) -> Result<Self> {
        let path: PathBuf = ["/proc", &pid.to_string(), "task", &tid.to_string(), "stat"]
            .into_iter()
            .collect();
        let name = format!(
            " - {}",
            get_process_command(tid).unwrap_or_else(|_| format!("PID {pid}"))
        );
        Self::with_capacity(path, name, cap)
    }

    pub fn sample(&mut self, start: Instant) -> Result<()> {
        if self.buf.len() >= self.buf.capacity() {
            self.buf.pop_front();
        }

        let new_sample = read_user_sys_time(&self.path)?;

        self.buf.push_back(SamplePoint {
            times: &new_sample - &self.last,
            duration: start.elapsed(),
        });

        self.last = new_sample;

        Ok(())
    }
}

struct ProcSampler {
    pid: usize,
    proc_buf: Buffer,
    thread_bufs: BTreeMap<usize, Buffer>,
    with_threads: bool,
}

impl ProcSampler {
    pub fn with_capacity(pid: usize, cap: usize, with_threads: bool) -> Result<Self> {
        let proc_buf = Buffer::from_pid(pid, cap)?;

        let thread_bufs = if with_threads {
            BTreeMap::from_iter(get_threads(pid).into_iter().filter_map(|tid| {
                Buffer::from_pid_tid(pid, tid, cap)
                    .ok()
                    .map(|buf| (tid, buf))
            }))
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            pid,
            proc_buf,
            thread_bufs,
            with_threads,
        })
    }

    pub fn sample(&mut self, start: Instant) -> Result<()> {
        // Sample main process
        self.proc_buf.sample(start)?;

        if self.with_threads {
            // Find potentially new threads
            let tids = get_threads(self.pid);
            for tid in tids {
                if let Entry::Vacant(e) = self.thread_bufs.entry(tid) {
                    if let Ok(buffer) =
                        Buffer::from_pid_tid(self.pid, tid, self.proc_buf.buf.capacity())
                    {
                        e.insert(buffer);
                    }
                }
            }

            let mut pids_to_remove = vec![];

            // Sample all threads
            for (pid, buffer) in self.thread_bufs.iter_mut() {
                if buffer.sample(start).is_err() {
                    pids_to_remove.push(*pid);
                }
            }

            // Remove unreadable threads
            for pid in pids_to_remove {
                self.thread_bufs.remove(&pid);
            }
        }

        Ok(())
    }

    pub fn buffers(&self) -> impl Iterator<Item = &Buffer> {
        [&self.proc_buf]
            .into_iter()
            .chain(self.thread_bufs.values())
    }
}

pub struct Sampler {
    start: Instant,
    proc_samplers: Vec<ProcSampler>,
}

impl Sampler {
    pub fn new(pids: &[usize], num_samples: usize, with_threads: bool) -> Result<Self> {
        let start = Instant::now();

        let proc_samplers = pids
            .iter()
            .map(|pid| ProcSampler::with_capacity(*pid, num_samples, with_threads))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            start,
            proc_samplers,
        })
    }

    pub fn sample(&mut self, sample_duration: Duration) -> Result<()> {
        let sleep = sample_duration.saturating_sub(Instant::now() - self.start);

        std::thread::sleep(sleep);

        for sampler in self.proc_samplers.iter_mut() {
            sampler
                .sample(self.start)
                .with_context(|| format!("failed to sample process with PID {}", sampler.pid))?;
        }

        self.start = Instant::now();

        Ok(())
    }

    pub fn buffers(&self) -> impl Iterator<Item = &Buffer> {
        self.proc_samplers.iter().flat_map(|s| s.buffers())
    }
}

/// Extract the user time and system time value from `/proc/../stat`.
fn read_user_sys_time(path: &Path) -> Result<ProcessTimes> {
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

/// Get the thread IDs of all threads spawned by this process
fn get_threads(pid: usize) -> Vec<usize> {
    fs::read_dir(format!("/proc/{}/task", pid))
        .unwrap()
        .filter_map(|res| {
            res.ok().and_then(|dir_entry| {
                dir_entry
                    .file_name()
                    .to_str()
                    .and_then(|tid| tid.parse::<usize>().ok())
            })
        })
        .filter(|tid| *tid != pid)
        .collect()
}
