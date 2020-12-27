use std::process;
use clap::{load_yaml, App, ArgMatches};
use walkdir::WalkDir;
use globwalk::GlobWalkerBuilder;
use std::fs::FileType;
use std::fs;

fn main() {

    // Load in our YAML file describing program options and extract user input
    let yams = load_yaml!("../dupe_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);

    println!("conf: {:?}", conf);

    let curr_dir = &conf.search_path[0];

    println!("Searching {}", curr_dir);

    let walker = WalkDir::new(curr_dir)
                                                     .min_depth(1)
                                        .into_iter()
                                        .filter_map(|e| e.ok());

    for en in walker {


        let clean_meta = en.clone();
        let clean_meta = clean_meta.metadata().unwrap();

        if clean_meta.is_file() {

            let clean_fn = en.clone();
            let clean_fn = clean_fn.path().display();

            let clean_ext = en.clone();
            //let clean_ext = clean_ext.into_path().extension().unwrap();


            let clean_size = clean_meta.len();

            //println!("File: {}\nExt: {}\nSize: {}\n==================",
            //            clean_fn, clean_ext.to_str().unwrap(), clean_size);
            println!("File: {}\nSize: {}\n==================",
                     clean_fn,  clean_size);
        }


    }

/*    let walker = GlobWalkerBuilder::from_patterns(
        curr_dir,
        &[conf.exts.as_str()]
    ).build();

    let walker_build = walker.unwrap();
    let walker_res = walker_build.into_iter();
//    let walker_map_res = walker_res.filter_map(Result::Ok);

    for f in walker_res {
        let g = f.unwrap();
        println!("curr f: {}", g);
    }*/



}

#[derive(Debug)]
struct Config {
    search_path: Vec<String>,
    archive : bool,
    debug   : bool,
    prog    : bool,
    resume  : bool,
    res_file : String,
    size     : String,
    jobs     : i32,

    work_file : String,
    hash_file : String,
    exts      : String
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
        let mut in_size = String::from("0 b");

        // Number of jobs to use
        let mut jobs = 1;

        // Extension string
        let mut exts = String::from("*");

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
            println!("Value for input_size: {}", in_size);
        }

        if let Some(in_exts) = in_args.value_of("exts") {
            exts = String::from(in_exts);
            //let paths = in_args.value_of("dir").unwrap();
            //let path_vec: Vec<String> = paths.split(',').map(|s| s.to_string()).collect();
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
            match n_jobs.parse::<i32>() {
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