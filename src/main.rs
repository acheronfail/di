// TODO: optionally print `n` largest files in scan

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

use structopt::StructOpt;

fn main() {
    let opt = cli::Opt::from_args();
    match scan::scan_dir(&opt) {
        Ok(result) => println!("{}", result),
        Err(e) => panic!(e),
    }
}
