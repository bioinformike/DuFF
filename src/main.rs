use std::process;
use clap::{load_yaml, App, ArgMatches};
//use walkdir::WalkDir;
//use globwalk::GlobWalkerBuilder;
use std::fs::FileType;
//use std::fs;
//use jwalk::WalkDirGeneric;
use rayon::ThreadPoolBuilder;

use crossbeam::crossbeam_channel;
use std::path::Path;
use crossbeam::queue;
use crossbeam::thread;
use std::fs::File;
use std::io::{BufReader, Read, Error};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use ring::digest::{Context, Digest, SHA256};
use walkdir::WalkDir;

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

    println!("conf: {:?}", conf);

    let mut file_res: Vec<FileResult> = vec![];

    use ignore::WalkBuilder;

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
        Box::new(move |result| {
            use ignore::WalkState::*;
            let curr_dir = match result {
                Ok(t) => t,
                Err(e) => {
                    println!("[Extract curr_dir error] {}", e);
                    return ignore::WalkState::Quit;
                }
            };

            let curr_path = curr_dir.path();
            let path_str = match curr_path.to_str() {
                Some(t) => t,
                None => {
                    println!("Error path-> path_str");
                    return ignore::WalkState::Quit;
                }
            };

            let path_str = String::from(path_str);

            let curr_meta = match curr_dir.metadata() {
                Ok(t) => t,
                Err(e) => {
                    println!("[Meta error] {}", e);
                    return ignore::WalkState::Quit;
                }
            };

            let fs = curr_meta.len();

            tx.send(FileResult::new(path_str, fs)).unwrap();
            Continue
        })
    });

    drop(tx);
    for t in rx.iter() {
        //let (sha, path) = t.unwrap();
        println!("{:?}", t);
    }
    drop(rx);
}






#[derive(Debug)]
struct Config {
    search_path: Vec<String>,
    archive : bool,
    debug   : bool,
    prog    : bool,
    resume  : bool,
    res_file : String,
    size     : u64,
    jobs     : u8,

    work_file : String,
    hash_file : String,
    exts      : Vec<String>
}

// struct to hold file information
#[derive(Debug)]
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

impl Config {
    pub fn new(in_args: ArgMatches)  -> Config {

        // Flags
        let mut is_arch = false;
        let mut is_debug = false;
        let mut is_prog  = false;
        let mut is_res = false;

        // File paths
        let mut res_file = String::from("");
        let mut hash_file = String::from("");
        let mut work_dir = String::from("");
        let mut in_size = 250_000;

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
        if let Some(in_size) = in_args.value_of("size") {
            let in_size = in_args.value_of("size").unwrap();
            let in_size = in_size.parse::<u64>();

        }

        if let Some(in_exts) = in_args.value_of("exts") {
            let in_exts = in_args.value_of("exts").unwrap();
            exts = in_exts.split(',').map(|s| s.to_string()).collect();
        }

        if let Some(res_f) = in_args.value_of("resume") {
            res_file = res_f.to_string();
            is_res = true;
        }

        if let Some(hash_f) = in_args.value_of("hash") {
            hash_file = hash_f.to_string();
        }

        if let Some(work_d) = in_args.value_of("work_dir") {
            work_dir = work_d.to_string();
        }

        if let Some(n_jobs) = in_args.value_of("jobs") {
            match n_jobs.parse::<u8>() {
                Ok(n) => jobs = n,
                Err(e) => {
                    println!("Number of jobs specificed, {}, is not a valid number!", n_jobs);
                    process::exit(1);

                },
            }
        }


        Config { search_path: path_vec, archive : is_arch, debug : is_debug, prog: is_prog,
            resume : is_res, res_file : res_file, size : in_size, jobs: jobs,
            work_file : work_dir, hash_file : hash_file, exts : exts }

    }
}