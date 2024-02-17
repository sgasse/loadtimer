use argh::FromArgs;

#[derive(FromArgs)]
/// Measure CPU usage of processes.
pub struct Cli {
    /// pid of process to monitor
    #[argh(positional)]
    pub pids: Vec<usize>,

    /// sample duration in seconds
    #[argh(option, short = 's', default = "30")]
    pub sample_secs: u64,

    /// break seconds
    #[argh(option, short = 'b', default = "0")]
    pub break_secs: u64,

    /// number of sample points
    #[argh(option, short = 'n', default = "2")]
    pub num_samples: usize,
}
