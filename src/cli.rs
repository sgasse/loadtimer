use argh::FromArgs;

#[derive(FromArgs)]
/// Measure CPU usage of processes.
pub struct Cli {
    /// PIDs of processes to measure
    #[argh(positional)]
    pub pids: Vec<usize>,

    /// sample duration in seconds
    #[argh(option, short = 's', default = "10")]
    pub sample_secs: u64,

    /// number of sample points
    #[argh(option, short = 'n', default = "2")]
    pub num_samples: usize,

    /// run in interactive mode
    #[argh(switch, short = 'i')]
    pub interactive: bool,
}
