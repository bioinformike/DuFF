use crate::util;

use std::{process, env, fs};
use pretty_bytes::converter;
use byte_unit::{Byte, ByteUnit};
use clap::ArgMatches;


#[derive(Debug, Clone)]
pub struct Config {

    pub search_path: Vec<String>,
    pub archive : bool,
    pub debug   : bool,
    pub prog    : bool,
    pub resume  : bool,
    pub have_hash : bool,
    pub user_set_dir : bool,
    pub res_file : String,
    pub ll_size     : u128,
    pub ul_size     : u128,
    pub jobs     : u64,

    pub work_dir : String,
    pub work_file : String,

    pub hash_file : String,
    pub prev_hash_file : String,

    pub report_file     : String,
    pub log_file : String,
    pub temp_file : String,

    pub exts      : Vec<String>
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


        // Files from previous work
        let mut prev_hash = String::from("");
        let mut res_file = String::from("");


        let mut work_dir = String::from("");

        // min and max size requirement, default for lower will just be 0, so we can use that
        // even if the user doesn't specify one.  Upper lim will use indicator of
        // 18446744073709551615 which is largest number u64 can hold and would be equal to  ~18EB
        // sooo we should not have to worry about this.
        let mut ll_size = 0;
        let mut ul_size = 340282366920938463463374607431768211455;

        // Number of jobs to use
        let mut jobs = 1;

        // Extension string
        let mut exts: Vec<String> = vec![String::from("*")];


        let paths = in_args.value_of("dir").unwrap();
        let path_vec: Vec<String> = paths.split(',').map(|s| s.to_string()).collect();

        // Check each dir handed in to make sure its accessible and a directory.
        for x in path_vec.iter() {
            match fs::metadata(x) {
                Ok(m) => {
                    if m.is_dir() == false {
                        let err_str = format!("Specified directory {} is not a directory!",
                                              x);
                        eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                        process::exit(1);
                    }
                },
                Err(e) => {
                    let err_str = format!("There was an error with the specified directory, {}: {}!",
                                          x, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                }
            }
        }

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
        if let Some(ll) = in_args.value_of("lower_lim") {
            match Byte::from_str(ll) {
                Ok(n) => ll_size = n.get_bytes(),
                Err(e) => {
                    let err_str = format!("Lower size limit {}: {}!",
                                          ll, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }

        // Max file size value
        if let Some(ul) = in_args.value_of("upper_lim") {
            match Byte::from_str(ul) {
                Ok(n) => ul_size = n.get_bytes(),
                Err(e) => {
                    let err_str = format!("Lower size limit {}: {}!",
                                          ul, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }


        if let Some(n_jobs) = in_args.value_of("jobs") {
            match n_jobs.parse::<u64>() {
                Ok(n) => jobs = n,
                Err(_e) => {
                    let err_str = format!("Number of jobs specificed, {}, is not a valid number!",
                                          n_jobs);
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
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
                              argument. \nError text: {}", util::PROG_NAME.to_owned() + util::PROG_VERS, e);
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));

                    std::process::exit(1);
                }
            };
            work_dir = String::from(cwd.to_str().unwrap())
        }


        // Specify the paths for our working files, we'll create them later.
        let work_file = format!("{}/DuFF_{}.working", work_dir, util::f_dt());
        let hash_file = format!("{}/DuFF_{}.hash", work_dir, util::f_dt());
        let log_file = format!("{}/DuFF_{}.log", work_dir, util::f_dt());
        let temp_file = format!("{}/DuFF_{}.temp", work_dir, util::f_dt());

        let report_file_str = format!("{}/DuFF_{}.report", work_dir, util::f_dt());
        let report_file = report_file_str.clone();

        Config {
            search_path: path_vec,
            exts: exts,

            ll_size: ll_size,
            ul_size: ul_size,
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

        let border_str = "=".repeat(textwrap::termwidth());
        println!("{}", border_str);
        println!("{:<21} {:^39} {:>0}", util::dt(), "Overview", util::PROG_NAME.to_owned() + " v" + util::PROG_VERS);
        println!("{}", border_str);
        println!("{:<40} {:>1}", "Status:", "New Run" );
        println!("{:<40} {:>1}", "Search Directories:", self.search_path.join(","));
        println!("{:<40} {:>1}", "Extensions:", self.exts.join(", "));

        // If they set a lower lim
        if self.ll_size > 0 {
            let ll_str = converter::convert(self.ll_size as f64);
            println!("{:<40} {:>1}", "Minimum Size:", ll_str );

        }
        // If they set a upper lim
        if self.ul_size < 340282366920938463463374607431768211455 {
            let ul_str = converter::convert(self.ul_size as f64);
            println!("{:<40} {:>1}", "Maximum Size:", ul_str);
        }

        println!("{:<40} {:>1}", "Number of Threads:", self.jobs);
        println!("{:<40} {:>1}", "Working Directory:", self.work_dir);
        println!("{:<40} {:>1}", "Final Report:", self.report_file);
        println!("{:<40} {:>1}", "Save Hashes:", self.archive);
        println!("{:<40} {:>1}", "Debug Mode:", self.debug);
        println!("{:<40} {:>1}", "Show Progress:", self.prog);
        println!("{:<40} {:>1}", "Report any issues at:", util::PROG_ISSUES);



    }
}