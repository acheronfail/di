use failure::Error;
use ignore::{WalkBuilder, WalkState};
use num_cpus;
use pretty_bytes;
use separator::Separatable;
use std::fmt;
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;

use cli;

#[derive(Debug)]
pub struct ScanResult {
    root: PathBuf,
    pub files: u64,
    pub directories: u64,
    pub symlinks: u64,
    pub bytes: u64,
}

impl ScanResult {
    pub fn new(root: PathBuf) -> ScanResult {
        ScanResult {
            root,
            files: 0,
            directories: 0,
            symlinks: 0,
            bytes: 0,
        }
    }
}

impl fmt::Display for ScanResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
{root}
    directories: {dirs}
    symlinks:    {symlinks}
    files:       {files}
    size:        {size} ({bytes} bytes)
",
            root = self.root.display(),
            files = self.files.separated_string(),
            dirs = self.directories.separated_string(),
            symlinks = self.symlinks.separated_string(),
            size = pretty_bytes::converter::convert(self.bytes as f64),
            bytes = self.bytes.separated_string()
        )
    }
}

pub fn scan_dir(opt: &cli::Opt) -> Result<ScanResult, Error> {
    let n_threads = opt.threads.unwrap_or(num_cpus::get());
    let root_dir = fs::canonicalize(&opt.root.as_ref().unwrap_or(&PathBuf::from(".")))?;

    if opt.verbosity > 1 {
        println!("directory: {}", root_dir.display());
        println!("number of threads: {}", n_threads);
    }

    let mut builder = WalkBuilder::new(&root_dir);
    let parallel_walker = builder
        .hidden(false)      // don't ignore hidden files
        .ignore(false)      // don't use .ignore files
        .git_ignore(false)  // don't use .gitignore files
        .git_exclude(false) // don't use .git/info/exclude files
        .threads(n_threads) // number of threads to use
        .build_parallel();

    let (tx, rx) = channel::<(PathBuf, fs::Metadata)>();
    let rx_thread = thread::spawn(move || {
        let mut scan_result = ScanResult::new(root_dir);
        let mut last_print = Instant::now();
        for (i, (_path, metadata)) in rx.into_iter().enumerate() {
            if metadata.is_file() {
                scan_result.bytes += metadata.len();
                scan_result.files += 1;
            } else if metadata.is_dir() {
                scan_result.directories += 1;
            } else {
                scan_result.symlinks += 1;
            }

            if last_print.elapsed().subsec_millis() >= 250 {
                print!("\rScanned {} entries...", i.separated_string());
                stdout().flush().unwrap();
                last_print = Instant::now();
            }
        }
        print!("\n");

        scan_result
    });

    parallel_walker.run(|| {
        // TODO: explain how this works
        let tx_thread = tx.clone();

        Box::new(move |entry_o| {
            let entry = match entry_o {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            match entry.metadata() {
                Ok(m) => {
                    // TODO: handle unwrap cleanly
                    tx_thread.send((entry.path().to_path_buf(), m)).unwrap();
                }
                Err(_) => return WalkState::Continue,
            }

            WalkState::Continue
        })
    });

    // TODO: explain how this works
    drop(tx);

    // TODO: explain how this works
    Ok(rx_thread.join().unwrap())
}
