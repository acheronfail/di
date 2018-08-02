use clap;
use std::path::PathBuf;

/// A tool to see where disk space is used
#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "di",
    after_help = "https://github.com/acheronfail/di",
    raw(setting = "clap::AppSettings::ColoredHelp")
)]
pub struct Opt {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,

    /// Set number threads (defaults to number of CPUs)
    #[structopt(short = "t", long = "threads")]
    pub threads: Option<usize>,

    /// Keep track of `n` largest files and directories
    #[structopt(short = "n", long = "number-of-items", default_value = "5")]
    pub n_items: usize,

    /// The directory to scan (defaults to current folder)
    #[structopt(name = "DIR", default_value = ".", parse(from_os_str))]
    pub root: PathBuf,
}
