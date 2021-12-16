// This file/module (I totally understand how Rust organizes code...) contains a bunch of utility
// functions used by the main logic.

// I sometimes use extra parens to make thing more readable to me
#![allow(unused_parens)]

// Both are required for process_file function
use crate::config::Config;
use crate::file_result;

// For our datetime helper functions (dt and f_dt)
use chrono::{DateTime, Utc};

// To create files for output (open_file) and clean up any they don't want to keep.
use std::fs::{File, remove_file};

// Paths are taken as input to 3 functions (open_file, check_ext, process_file)
use std::path::{Path, PathBuf};

// For writing out our report file.
use std::io::Write;

// For printing out report
use std::collections::HashMap;

// For the find task function
use crossbeam_deque::{Injector, Worker, Steal};
use crate::file_result::FileResult;


// Extract some info from our manifest file to be used at different places for output to user.
pub const PROG_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const PROG_VERS: &'static str = env!("CARGO_PKG_VERSION");
pub const PROG_ISSUES: &'static str = "https://github.com/bioinformike/DuFF/issues";


// Simple date-timestamp function that just returns the current date and time in following format:
// 2020-12-31 14:55:06.  This is used in output to user and to files.
pub fn dt() -> String {
    let now: DateTime<Utc> = Utc::now();
    String::from(format!("{}", now.format("%Y-%m-%d %H:%M:%S")))
}

// Date-timestamp with hyphens replaced by underscores for easier reading in filenames
// 2020_12_31__14_55_06. This function is used for including date and time in filenames.
pub fn f_dt() -> String {
    let now: DateTime<Utc> = Utc::now();
    String::from(format!("{}", now.format("%Y_%m_%d__%H_%M_%S")))
}


// The open_file function is a helper function that simply creates a file and returns that created
// file to the caller.
// Arguments are as follows:
// file_str: Full path of the file to create.
// out_dir:  The current output directory only used in error messages.
// user_dir: Bool to let function know if the user specified the output directory or we tried to use
//           the default (current working directory). This changes the error message, in an attempt
//           to be more helpful.
pub fn open_file(file_str: &String, out_dir: &String, user_dir: bool) -> File {

    // Try to create the specified file and catch any errors.
    let new_file = match File::create(Path::new(&file_str)) {
        Ok(f) => f,
        Err(e) => {

            // If the user specified the output directory, let them know this didn't work.
            if user_dir {
                let err_str = format!("Could not write to specified working directory {}.\n\
                    Please specify a working directory with write permissions where {} can write the \
                    final report and any other requested output files using the -o (--out) argument.\
                    \nError text: {}", out_dir, PROG_NAME.to_owned() + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));

            // User didn't specify an output directory, so we tried CWD, but that failed.
            } else {
                let err_str = format!("You did not specify an output directory (-o, --out) \
                    and the CWD [{}] is not writeable.\nPlease specify where {} can write the final \
                    report and any other requested output files using the -o (--out) argument.\n\
                    Error text: {}", out_dir, PROG_NAME.to_owned() + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
            }

            // Kill the program
            std::process::exit(1);
        },
    };
    return new_file
}

// The check_ext function checks to see if the current file has an extension that matches one
// specified by the user or not and returns a bool indicating the result.
// Arguments are as follows:
// curr_file: The current file to check
// curr_exts: The list of extensions we should check for.
pub fn check_ext(curr_file: &Path, curr_exts: &Vec<String>) -> bool {

    // If we only have 1 extension and that extension is the default asterisk, go ahead and return
    // true
    if ((curr_exts.len() == 1) && (curr_exts[0] == "*")) {
        return true;
    } else {

        // Get a string version of the filename
        // TODO: Add functionality to replace these 2 unwraps!
        let clean_fn = curr_file.file_name().unwrap().to_str().unwrap().to_string();

        // Loop over the extensions list and see if our file (curr_file) ends with one of the
        // extensions.  Theoretically, this way of dealing with extensions should mean the user
        // does not have to (but still can) include dots prior to the extension, although they must
        // include the dots if they are inside an extension, like a .fastq.gz.
        for x in curr_exts.iter() {

            // If there is match return true immediately
            if clean_fn.ends_with(x) {
                return true;
            }
        }

        // If no extension match was found, return false.
        return false;
    }
}

// The check_size function checks to see if the current file matches the size requirements that the
// user may have specified (the function is still run even when the user did not specify a lower or
// upper limit, in this case check_size just receives the default values for ll_size and ul_size)
// returning a bool indicating whether or not it does. This could have been left in the main code
// but pulling it out seemed cleaner to me.
// Arguments are as follows:
// curr_fs: The file size of whatever file being checked.
// min_size: The lower limit file size that curr_fs must be >= than to be further considered.
// max_size: The upper limit file size that curr_fs must be <= than to be further considered.
pub fn check_size(curr_fs: u128, min_size: u128, max_size: u128) -> bool {

    // Make sure curr_fs is gt or eq to min_size(default = 0B) AND is lt or eq to max_size (default
    // = ~340 Yottabytes)
    ((curr_fs >= min_size) & (curr_fs <= max_size))

}


// The find_task function is "adapted" from Crossbeam's deque docs
// [https://docs.rs/crossbeam/0.7.1/crossbeam/deque/index.html] and from Ken Sternberg's Parallel
// Boggle Solver cited above.
pub fn find_task<T>(local: &mut Worker<T>, global: &Injector<T>) -> Option<T> {
    match local.pop() {
        Some(job) => Some(job),
        None => loop {
            match global.steal() {
                Steal::Success(job) => break Some(job),
                Steal::Empty => break None,
                Steal::Retry => {}
            }
        },
    }
}


// This function does all the processing of a PathBuf. Specifically, it collects the metadata
// (filesize and mtime) and will create a new FileResult object which it will return wrapped in a
// Some, otherwise if this function hits an error or the path in question doesn't satisfy the
// extension or file size requirements None is returned.
pub fn process_file(curr_pb: &PathBuf, curr_conf: &Config) -> Option<file_result::FileResult> {

    // Convert pathbuf to just path
    let mut curr_path = curr_pb.as_path();

    // And canonicalize the path
    let canon_path = match curr_path.canonicalize() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error canonicalizing path!\n{}", e);
            return None;
        }
    };

    // Extract the path version
    curr_path = canon_path.as_path();

    // Now try to extract the file name as OsStr and then convert that to actual str
    let name_os = match curr_path.file_name() {
        Some(u) => u,
        None => {
            eprintln!("Error extracting file name from path.");
            return None;
        }
    };

    // Convert the OsStr to str
    let name_str = match name_os.to_str() {
        Some(u) => u,
        None => {
            eprintln!("Error converting file name OsStr to str.");
            return None;
        }
    };

    let file_name = String::from(name_str);

    // Get the path to this file
    let dir_path = match canon_path.parent() {
        Some(u) => u,
        None => {
            eprintln!("Error extracting directory path!");
            return None;
        }
    };

    // Capture this path as a str
    let dir_str = match dir_path.to_str() {
        Some(u) => u,
        None => {
            eprintln!("Error converting path to string.");
            return None;
        }
    };

    let dir_str = String::from(dir_str);

    // Capture this path as a str
    let path_str = match curr_path.to_str() {
        Some(u) => u,
        None => {
            eprintln!("Error converting path to string.");
            return None;
        }
    };

    let path_str = String::from(path_str);

    // Attempt to get the metadata for this file so we can access m-time and file size
    let curr_meta = match curr_path.metadata() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error collecting metadata. {}", e);
            return None;
        }
    };

    // Grab m-time and convert to UTC time
    let mtime = match curr_meta.modified() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error capturing mtime for file. {}", e);
            return None;
        }
    };

    let mtime: chrono::DateTime<Utc> = mtime.into();

    // Grab the file size
    let fs = u128::from(curr_meta.len());

    // Skip files of size 0
    if fs == 0 {
        return None
    }

    // Run our extension and size matching checks based on user's input
    let ext_match = check_ext(curr_path, &curr_conf.exts);
    let size_match = check_size(fs,
                                      curr_conf.ll_size,
                                      curr_conf.ul_size);

    // As long as this file fits the user's requirements return a FileResult struct, if not just
    // return a None.
    if ext_match && size_match {
        return Some(file_result::FileResult::new(file_name, dir_str,path_str, fs, mtime));
    }

    return None
}

// This function does all the end of processing cleaning up.  Saving some files if the user wanted
// them and deleting them if they didn't.
pub fn clean_up(curr_conf: &Config)  {

    // Remove the log if they never asked for a log (log bool == False)
    if !curr_conf.log {
        let log_path = Path::new(&curr_conf.log_file);

        match remove_file(log_path) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error while trying to remove log file {}.\nError text: {}",
                           &curr_conf.log_file, e);
            }
        }
    }

    // Remove the archive if they never asked for an archive (archive bool == False)
    if !curr_conf.archive {
        let arch_path = Path::new(&curr_conf.archive_file);

        match remove_file(arch_path) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error while trying to remove log file {}.\nError text: {}",
                          &curr_conf.archive_file, e);
            }
        }
    }
}

// This function writes a report file out to the file represented by rep_file. It iterates through
// all of the duplicate files in the input dict making entries for each one.
pub fn write_report<T>(mut rep_file: File, dict: HashMap<T, Vec<FileResult>>) {

    // TODO: Replace unwrap
    // Write the simple header
    writeln!(rep_file, "File Count\tDuplicate Number\tName\tPath\tFile Size\tModified Time").unwrap();


    // file_cnt tracks the number of unique files (files that have multiple copies)
    let mut file_cnt = 1;

    // Go through th entire dictionary
    for (_, v) in dict.iter() {

        // Create string we will build on
        let mut out_str = String::new();


        // dupe_cnt represents the number duplicate of current file_cnt this file is
        let mut dupe_cnt = 1;

        // Go through each vector of duplicate files
        for y in v.iter() {

            // Append information for current duplicate to our string for output
            out_str.push_str(format!("{}\t{}\t{}\t{}\t{}\t{}\n", file_cnt, dupe_cnt,
                                     y.file_name, y.dir_path, y.size, y.mtime).as_str());


            dupe_cnt = dupe_cnt + 1;
        }

        // Write out the report entry for this unique file.
        write!(rep_file, "{}", out_str).unwrap();


        file_cnt = file_cnt + 1;
    }
}

