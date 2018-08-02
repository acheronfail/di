use ansi_term::{ANSIStrings, Colour, Style};
use failure::Error;
use ignore::{WalkBuilder, WalkState};
use num_cpus;
use pretty_bytes;
use separator::Separatable;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Instant;
use std::{fmt, fs, thread};

use cli;
use util::{Info, LimitedHeap};

#[derive(Debug)]
pub struct ScanResult {
    // The root path to scan.
    pub root: PathBuf,
    // Statistic of entries scanned.
    pub files: u64,
    pub directories: u64,
    pub symlinks: u64,
    // Total size of files in bytes.
    pub bytes: u64,

    pub largest_dirs: LimitedHeap,
    pub largest_files: LimitedHeap,
}

impl ScanResult {
    pub fn new(root: PathBuf, n_files: usize) -> ScanResult {
        ScanResult {
            root,
            files: 0,
            directories: 0,
            symlinks: 0,
            bytes: 0,
            largest_dirs: LimitedHeap::new(n_files),
            largest_files: LimitedHeap::new(n_files),
        }
    }
}

impl fmt::Display for ScanResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let grey = Colour::Fixed(244);
        let green = Colour::Green;
        let yellow = Colour::Yellow;
        let underline = Style::new().underline();

        write!(
            f,
            "{} {}\n\n",
            underline.fg(Colour::Cyan).paint(env!("CARGO_PKG_NAME")),
            grey.paint(self.root.display().to_string())
        )?;
        write!(f, "{}\n", underline.paint("Scan statistics:"))?;
        write!(
            f,
            " {}\n",
            ANSIStrings(&[
                grey.paint("directories   "),
                green.paint(self.directories.separated_string())
            ])
        )?;
        write!(
            f,
            " {}\n",
            ANSIStrings(&[
                grey.paint("symlinks      "),
                green.paint(self.symlinks.separated_string())
            ])
        )?;
        write!(
            f,
            " {}\n",
            ANSIStrings(&[
                grey.paint("files         "),
                green.paint(self.files.separated_string())
            ])
        )?;
        write!(
            f,
            " {}\n",
            ANSIStrings(&[
                grey.paint("total entries "),
                green.paint((self.files + self.directories + self.symlinks).separated_string())
            ])
        )?;
        write!(
            f,
            " {}\n",
            ANSIStrings(&[
                grey.paint("total size    "),
                yellow.paint(pretty_bytes::converter::convert(self.bytes as f64)),
                grey.paint(" ("),
                yellow.paint(self.bytes.separated_string()),
                grey.paint(" bytes)")
            ])
        )?;
        write!(
            f,
            "\n{}\n{}\n\n{}\n{}",
            underline.paint("Largest directories found:"),
            underline.paint("Largest files found:"),
            self.largest_dirs,
            self.largest_files
        )?;

        Ok(())
    }
}

pub fn scan_dir(opt: &cli::Opt) -> Result<ScanResult, Error> {
    let n_threads = opt.threads.unwrap_or(num_cpus::get());
    let root_dir = fs::canonicalize(&opt.root)?;

    if opt.verbosity > 0 {
        println!("scanning directory: {}", root_dir.display());
        println!("number of threads:  {}", n_threads);
    }

    let mut builder = WalkBuilder::new(&root_dir);
    let parallel_walker = builder
        .hidden(false)      // don't ignore hidden files
        .ignore(false)      // don't use .ignore files
        .git_ignore(false)  // don't use .gitignore files
        .git_exclude(false) // don't use .git/info/exclude files
        .threads(n_threads) // number of threads to use
        .build_parallel();

    let rx_opt = opt.clone();
    let (tx, rx) = channel::<(PathBuf, fs::Metadata)>();
    let rx_thread = thread::spawn(move || {
        let mut dir_map = HashMap::<String, u64>::new();
        let mut scan_result = ScanResult::new(root_dir, rx_opt.n_items);
        let mut last_print = Instant::now();
        for (i, (mut path, metadata)) in rx.into_iter().enumerate() {
            if metadata.is_file() {
                scan_result.files += 1;

                let bytes = metadata.len();
                scan_result.bytes += bytes;
                scan_result.largest_files.push(Info(bytes, path.clone()));

                if let Some(parent) = path.parent() {
                    let key = format!("{}", parent.display());
                    let value = dir_map.entry(key).or_insert(0);
                    *value += bytes;
                }
            } else if metadata.is_dir() {
                scan_result.directories += 1;
            } else {
                scan_result.symlinks += 1;
            }

            if rx_opt.verbosity > 1 && last_print.elapsed().subsec_millis() >= 250 {
                print!("\rScanned {} entries...", (i + 1).separated_string());
                stdout().flush().unwrap();
                last_print = Instant::now();
            }
        }
        print!("\r");

        // TODO: rather than just finding the largest directories, use this data
        // in a more useful way?
        for (path_str, bytes) in dir_map {
            scan_result
                .largest_dirs
                .push(Info(bytes, PathBuf::from(path_str)));
        }

        scan_result
    });

    parallel_walker.run(|| {
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

    // Drop the initial sender. If we don't do this, the receiver will block
    // even if all threads have finished, since there is still one sender around.
    drop(tx);

    // Wait for the receiver thread finish.
    Ok(rx_thread.join().unwrap())
}
