use std::fs;

use anyhow::Result;
use nix::unistd::SysconfVar;

pub mod cli;
pub mod dump;
pub mod eval;
pub mod sample;

/// Get the `USER_HZ` constant, describing the number of jiffies.
///
/// The user and system time values reported in `/proc/{pid}/stat` do not correspond to actual
/// cycles but to virtual cycles. The constant `USER_HZ` describes how many virtual cycles there
/// are in one second.
pub fn get_user_hz() -> Result<i64> {
    nix::unistd::sysconf(SysconfVar::CLK_TCK)?.ok_or(anyhow::anyhow!(
        "failed to retreive USER_HZ / SC_CLK_TCK / jiffies"
    ))
}

/// Get the command behind a process.
pub fn get_process_command(pid: usize) -> Result<String> {
    let path = format!("/proc/{pid}/comm");
    fs::read_to_string(path).map_err(Into::into)
}
