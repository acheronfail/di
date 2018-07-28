// TODO: calculate and show largest directories (like windirstat, etc)
// TODO: display percentages as a part of results

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
        Ok(scan_result) => println!("\n{}", scan_result),
        Err(e) => panic!(e),
    }
}
