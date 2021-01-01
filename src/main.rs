use std::{process, fmt};
use clap::{load_yaml, App, ArgMatches};
use chrono::{DateTime, Utc};


use crossbeam::crossbeam_channel;
use std::path::Path;
use walkdir::WalkDir;
use ignore::WalkBuilder;
use std::env;
use textwrap;
use pretty_bytes::converter;

use ring::digest::{Context, Digest, SHA256};
use std::fs::File;

// Extract some info from our manifest
const PROG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PROG_VERS: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const PROG_HOME: &'static str = env!("CARGO_PKG_HOMEPAGE");
const PROG_ISSUES: &'static str = "https://github.com/bioinformike/dupe_finder/issues";


// Simple date-timestamp function, just returns date and time in following format:
// [2020-12-31 14:55:06]
// Includes the square brackets
pub fn dt()  -> String {
    let now: DateTime<Utc> = Utc::now();
    String::from(format!("[{}]", now.format("%Y-%m-%d %H:%M:%S")))
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
fn open_file(file_str: &String, work_dir: &String, user_dir: bool) -> File {
    let mut new_file = match File::create(Path::new(&file_str)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if user_dir {
                let err_str = format!("Could not write to specified working directory {}.  \nPlease specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument. Error text: {}", work_dir,
                              PROG_NAME.to_owned() + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(),textwrap::termwidth()));

                // User didn't give us a directory so we tried cwd.
            } else {
                let err_str = format!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.Error \
                               text: {}", work_dir, PROG_NAME.to_owned()  + " v" + PROG_VERS, e);
                println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
            }
            // Kill the program
            std::process::exit(1);
        },
    };
    new_file
}

fn is_good_ext(curr_dir: &Path, curr_exts: &Vec<String>) -> bool {

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
fn is_good_size(curr_fs: u64, min_size: u64) -> bool {
    curr_fs > min_size
}




fn main() {

    // Get user input
    let yams = load_yaml!("../dupe_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);


    let mut work_file = open_file(&conf.work_file, &conf.work_dir, conf.user_set_dir);
    // Try creating our files and if we can't tell the user that we don't have the write
    // permissions we need for either the directory they specified or cwd
    let mut work_file = match File::create(Path::new(&conf.work_file)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if conf.user_set_dir {
                println!("Could not write to specified working directory {}.  Please specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument.\nError text: {}", conf.work_dir,
                              PROG_NAME.to_owned() + PROG_VERS, e);

                // User didn't give us a directory so we tried cwd.
            } else {
                println!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.\nError \
                               text: {}", conf.work_dir, PROG_NAME.to_owned() + PROG_VERS, e);
            }
            // Kill the program
            std::process::exit(1);
        },
    };

    let mut hash_file = match File::create(Path::new(&conf.hash_file)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if conf.user_set_dir {
                println!("Could not write to specified working directory {}.  Please specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument.\nError text: {}", conf.work_dir,
                         PROG_NAME.to_owned() + PROG_VERS, e);

                // User didn't give us a directory so we tried cwd.
            } else {
                println!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.\nError \
                               text: {}", conf.work_dir, PROG_NAME.to_owned() + PROG_VERS, e);
            }
            // Kill the program
            std::process::exit(1);
        },
    };

    let mut log_file = match File::create(Path::new(&conf.log_file)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if conf.user_set_dir {
                println!("Could not write to specified working directory {}.  Please specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument.\nError text: {}", conf.work_dir,
                         PROG_NAME.to_owned() + PROG_VERS, e);

                // User didn't give us a directory so we tried cwd.
            } else {
                println!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.\nError \
                               text: {}", conf.work_dir, PROG_NAME.to_owned() + PROG_VERS, e);
            }
            // Kill the program
            std::process::exit(1);
        },
    };

    let mut temp_file = match File::create(Path::new(&conf.temp_file)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if conf.user_set_dir {
                println!("Could not write to specified working directory {}.  Please specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument.\nError text: {}", conf.work_dir,
                         PROG_NAME.to_owned() + PROG_VERS, e);

                // User didn't give us a directory so we tried cwd.
            } else {
                println!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.\nError \
                               text: {}", conf.work_dir, PROG_NAME.to_owned() + PROG_VERS, e);
            }
            // Kill the program
            std::process::exit(1);
        },
    };

    let mut report_file = match File::create(Path::new(&conf.report_file)) {
        Ok(f) => f,
        Err(e) => {
            // If the user specified the working dir
            if conf.user_set_dir {
                println!("Could not write to specified working directory {}.  Please specify \
                              a working directory with write permissions where {} can store \
                              temporary files and the final report using the -f (--file) \
                              argument.\nError text: {}", conf.work_dir,
                         PROG_NAME.to_owned() + PROG_VERS, e);

                // User didn't give us a directory so we tried cwd.
            } else {
                println!("You did not specify a working directory (-f, --file) and the CWD\
                               [{}] is not writeable. Please specify where {} can store temporary \
                               files and the final report using the -f (--file) argument.\nError \
                               text: {}", conf.work_dir, PROG_NAME.to_owned() + PROG_VERS, e);
            }
            // Kill the program
            std::process::exit(1);
        },
    };

    conf.print();

    let mut file_res: Vec<FileResult> = vec![];

    let (tx, rx) = crossbeam_channel::unbounded::<FileResult>();

    let mut dirs = conf.search_path.clone();
    let curr_dir = dirs.pop().unwrap();

    let mut walker = ignore::WalkBuilder::new(Path::new(curr_dir.as_str()));

    for x in dirs.iter() {
        walker.add(Path::new(x.as_str()));
    }
    walker.threads(conf.jobs as usize);
    let walker = walker.build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        let conf = conf.clone();
        Box::new(move |result| {
            use ignore::WalkState::*;
            let curr_dir = match result {
                Ok(t) => t,
                Err(e) => {
                    println!("[Extract curr_dir error] {}", e);
                    return ignore::WalkState::Continue;
                }
            };

            let curr_path = curr_dir.path();

            // We don't care abou directories!
            if curr_path.is_dir() {
                return ignore::WalkState::Continue;
            }

            let path_str = match curr_path.to_str() {
                Some(t) => t,
                None => {
                    println!("Error path-> path_str");
                    return ignore::WalkState::Continue;
                }
            };

            let path_str = String::from(path_str);

            let curr_meta = match curr_dir.metadata() {
                Ok(t) => t,
                Err(e) => {
                    println!("[Meta error] {}", e);
                    return ignore::WalkState::Continue;
                }
            };

            let fs = curr_meta.len();

            // only want to send something down the channel if its a file and meets our extension
            // and size requirements.
            let ext_match = is_good_ext(curr_path, &conf.exts);
            let size_match = is_good_size(fs, conf.size);
            if ext_match && size_match  {
                tx.send(FileResult::new(path_str, fs)).unwrap();
            }

            Continue
        })
    });

    drop(tx);
    let rx_iter = rx.iter();
    println!("Rxiter {}", rx_iter.count());
    for t in rx.iter() {
        //let (sha, path) = t.unwrap();
        println!("{:?}", t);
    }
    drop(rx);
}



// struct to hold file information
#[derive(Debug, Clone)]
struct FileResult {
    file_path : String,
    size : u64,
    hash : String
}

impl FileResult {
    pub fn new(file_path: String, size: u64) -> FileResult {
        FileResult {file_path, size, hash : String::new()}
    }

    pub fn update_hash(&mut self, hash: String)  {
        self.hash = hash;
    }
}




#[derive(Debug, Clone)]
struct Config {
    search_path: Vec<String>,
    archive : bool,
    debug   : bool,
    prog    : bool,
    resume  : bool,
    have_hash : bool,
    user_set_dir : bool,
    res_file : String,
    size     : u64,
    jobs     : u64,

    work_dir : String,
    work_file : String,

    hash_file : String,
    prev_hash_file : String,

    report_file     : String,
    log_file : String,
    temp_file : String,

    exts      : Vec<String>
}


impl Config  {
    pub fn new(in_args: ArgMatches) -> Config {

        // Flags
        let mut is_arch = false;
        let mut is_debug = false;
        let mut is_prog = false;
        let mut is_res = false;
        let mut have_precomp_hash = false;
        let mut user_set_dir = false;

        let mut prev_hash = String::from("");

        let mut res_file = String::from("");
        let mut hash_file = String::from("");
        let mut report_file = String::from("");

        let mut work_dir = String::from("");

        // min size requirement, default 250MB = 250,000,000 Bytes
        let mut in_size = 250_000_000;

        // Number of jobs to use
        let mut jobs = 1;

        // Extension string
        let mut exts: Vec<String> = vec![String::from("*")];

        let paths = in_args.value_of("dir").unwrap();
        let path_vec: Vec<String> = paths.split(',').map(|s| s.to_string()).collect();

        // Deal with our flag options
        if in_args.is_present("archive") {
            is_arch = true;
        }

        if in_args.is_present("debug") {
            is_debug = true;
        }

        if in_args.is_present("prog") {
            is_prog = true;
        }

        // Deal with arguments

        // Minimum size specification in bytes!
        if let Some(in_size) = in_args.value_of("size") {
            let in_size = in_args.value_of("size").unwrap();
            let in_size = in_size.parse::<u64>();
        }

        if let Some(n_jobs) = in_args.value_of("jobs") {
            match n_jobs.parse::<u64>() {
                Ok(n) => jobs = n,
                Err(e) => {
                    let err_str = format!("Number of jobs specificed, {}, is not a valid number!",
                                      n_jobs);
                    println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }


        // Extensions to search for, comma separated values.
        if let Some(in_exts) = in_args.value_of("exts") {
            let in_exts = in_args.value_of("exts").unwrap();
            exts = in_exts.split(',').map(|s| s.to_string()).collect();
        }

        // If they want us to resume then they need to give us a resume file that contains
        // all search results and we will skip directly to sorting, finding size dupes, then hashing
        if let Some(res_f) = in_args.value_of("resume") {
            res_file = res_f.to_string();
            is_res = true;
        }

        // If they have precomputed hash file we will load this in and skip any files in the hash
        // file.
        if let Some(hash_f) = in_args.value_of("hash") {
            prev_hash = hash_f.to_string();
            have_precomp_hash = true;
        }

        // Get work directory, if the user doesn't give us one we try to use cwd and if that fails
        // we quit!
        if let Some(work_d) = in_args.value_of("work_dir") {
            work_dir = work_d.to_string();
            user_set_dir = true;
        } else {
            // File paths
            let cwd = match env::current_dir() {
                Ok(t) => t,
                Err(e) => {
                    let err_str = format!("Could not use current working directory, please specify where {} can \
                              store temporary files and the final report using the -f (--file) \
                              argument. \nError text: {}", PROG_NAME.to_owned() + PROG_VERS, e);
                    println!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));

                    std::process::exit(1);
                }
            };
            work_dir = String::from(cwd.to_str().unwrap())
        }


        // Specify the paths for our working files, we'll create them later.
        let mut work_file = format!("{}/wl_dupe_finder_{}.working", work_dir, f_dt());
        let mut hash_file = format!("{}/wl_dupe_finder_{}.hash", work_dir, f_dt());
        let mut log_file = format!("{}/wl_dupe_finder_{}.log", work_dir, f_dt());
        let mut temp_file = format!("{}/wl_dupe_finder_{}.temp", work_dir, f_dt());

        let mut report_file_str = format!("{}/wl_dupe_finder_{}.report", work_dir, f_dt());
        let mut report_file = report_file_str.clone();

        Config {
            search_path: path_vec,
            exts: exts,

            size: in_size,
            jobs: jobs,

            archive: is_arch,
            debug: is_debug,
            prog: is_prog,
            user_set_dir : user_set_dir,

            resume: is_res,
            res_file: res_file,

            work_dir: work_dir,
            work_file: work_file,

            hash_file: hash_file,
            have_hash: have_precomp_hash,
            prev_hash_file: prev_hash,

            report_file: report_file,
            log_file: log_file,
            temp_file: temp_file
        }
    }

    pub fn print(&self) {
        let size_str = converter::convert(self.size as f64);

        let border_str = "=".repeat(textwrap::termwidth());
        println!("{}", border_str);
        println!("{:<21} {:^39} {:>0}", dt(), "Overview", PROG_NAME.to_owned() + " v" + PROG_VERS);
        println!("{}", border_str);
        println!("{:<40} {:>1}", "Status:", "New Run" );
        println!("{:<40} {:>1}", "Search Directories:", self.search_path.join(","));
        println!("{:<40} {:>1}", "Extensions:", self.exts.join(", "));
        println!("{:<40} {:>1}", "Size Threshold:", size_str );
        println!("{:<40} {:>1}", "Number of Threads:", self.jobs);
        println!("{:<40} {:>1}", "Working Directory:", self.work_dir);
        println!("{:<40} {:>1}", "Final Report:", self.report_file);
        println!("{:<40} {:>1}", "Save Hashes:", self.archive);
        println!("{:<40} {:>1}", "Debug Mode:", self.debug);
        println!("{:<40} {:>1}", "Show Progress:", self.prog);
        println!("{:<40} {:>1}", "Report any issues at:", PROG_ISSUES);



    }
}

      /*  // This is the most horrendous thing I've ever written, but this seemed like the most
        // straightforward way, somehow, please forgive me!


        let stat_str = "Status:             New Run";
        let search_str  = format!("Search Directories: {}",
                                  self.search_path.join(", "));
        let ext_str = format!("Extensions:            {}",self.exts.join(", "));
        let size_str = format!("Size Threshold:    >{} Bytes", self.size);
        let thread_str = format!("Number of Threads:         {}", self.jobs);
        let work_str = format!("Working Directory:            {}", self.work_dir);
        let report_str = format!("Final Report:             {}", self.res_file);
        let hash_str = format!("Save Hashes:             {}", self.archive);
        let debug_str = format!("Debug Mode:              {}", self.debug);
        let prog_str = format!("Show Progress:             {}", self.prog);
        let issue_str = format!("Report any issues at {}", PROG_ISSUES);


        write!(f, "{border}\n{over}\n{border}\n{search}\n{ext}\n{size}\n{thread}\n{work}\n\
                     {report}\n{hash}\n{debug}\n{prog}\n{issue}",

                     border = border_str, over = over_str, search = search_str, ext = ext_str,
                     size = size_str, thread = thread_str, work = work_str, report = report_str,
                     hash = hash_str, debug = debug_str, prog = prog_str, issue = issue_str)



    }*/
