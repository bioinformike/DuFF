/*
    This file/module contains a bunch of utility functions used by the main logic.
 */

// I sometimes use extra parens to make thing more readable to me
#![allow(unused_parens)]

// Load in required crates
use chrono::{DateTime, Utc};
use std::fs::File;
use std::path::Path;

// Extract some info from our manifest
pub const PROG_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const PROG_VERS: &'static str = env!("CARGO_PKG_VERSION");
pub const PROG_ISSUES: &'static str = "https://github.com/bioinformike/DuFF/issues";


// Simple date-timestamp function, just returns date and time in following format:
// [2020-12-31 14:55:06]
// Includes the square brackets
pub fn dt() -> String {
    let now: DateTime<Utc> = Utc::now();
    String::from(format!("{}", now.format("%Y-%m-%d %H:%M:%S")))
}

// Date-timestamp with hyphens replaced with underscores for easier reading in filenames
// 2020_12_31__14_55_06
pub fn f_dt() -> String {
    let now: DateTime<Utc> = Utc::now();
    String::from(format!("{}", now.format("%Y_%m_%d__%H_%M_%S")))
}

// user_dir: User specified the directory to use (changes the error message)
// file_str: full path to file to be created.
// work_dir: the path to the directory where the file is being placed (used for error messages)
pub fn open_file(file_str: &String, work_dir: &String, user_dir: bool) -> File {
    let new_file = match File::create(Path::new(&file_str)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if user_dir {
                let err_str = format!("Could not write to specified working directory {}.  \nPlease specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument. Error text: {}", work_dir,
                                      PROG_NAME.to_owned() + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));

                // User didn't give us a directory so we tried cwd.
            } else {
                let err_str = format!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.Error \
                               text: {}", work_dir, PROG_NAME.to_owned() + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
            }
            // Kill the program
            std::process::exit(1);
        },
    };
    new_file
}

// Checks if the input Path matches one of the extensions in the curr_exts input.
// If curr_exts only has one element and that is '*', then this function returns true stat.
pub fn is_good_ext(curr_dir: &Path, curr_exts: &Vec<String>) -> bool {
    if ((curr_exts.len() == 1) && (curr_exts[0] == "*")) {
        return true;
    } else {
        let clean_fn = curr_dir.file_name().unwrap().to_str().unwrap().to_string();

        for x in curr_exts.iter() {
            if clean_fn.ends_with(x) {
                return true;
            }
        }

        return false;
    }
}

// Checks if the size of the file input as curr_fs is greater than or equal to the requested
// minimum file size.  This could have been left in the main code but pulling it out seemed
// cleaner to me.
pub fn is_good_size(curr_fs: u128, min_size: u128, max_size: u128) -> bool {

    // Make sure curr_fs is gt or eq to min_size(default = 0B) AND is lt or eq to max_size (default
    // = ~18EB)
    ((curr_fs >= min_size) & (curr_fs <= max_size))

}

/*
pub fn scan_fs() {
    crossbeam::scope(|spawner| {
        let handles: Vec<ScopedJoinHandle<Vec<String>>> = (0..conf.jobs)
            .map(|_| {
                spawner.spawn(move |_| {
                    let mut local_q: Worker<Job> = Worker::new_fifo();

                    loop {
                        match find_task(&mut local_q, &global_q) {
                            Some(mut job) => {

                                // This should really only be the case, but we'll handle
                                // if its a file too.
                                if job.is_dir() {
                                    println!("Dir: {}", job.to_str().unwrap());
                                } else if job.is_file() {
                                    println!("File: {}", job.to_str().unwrap());
                                }
                            }
                            None => break,
                        }
                    }
                })
            })
            .collect();
    }
}
*/
// Reads contents of directory and pushes result down channel
pub fn read_dir(dir: &Path)  {

}