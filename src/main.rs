/// IDEA: use ncurses and save directory information to browse down fs and see where disk space is used
/// IDEA: display file sizes as percentages of total space used
extern crate ansi_term;
extern crate clap;
extern crate failure;
extern crate ignore;
extern crate num_cpus;
extern crate pretty_bytes;
extern crate separator;
#[macro_use]
extern crate structopt;

mod cli;
mod scan;
mod util;

use structopt::StructOpt;

fn main() {
    let opt = cli::Opt::from_args();
    match scan::scan_dir(&opt) {
        Ok(scan_result) => println!("{}", scan_result),
        Err(e) => panic!(e),
    }
}
