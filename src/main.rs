use std::process;
use clap::{load_yaml, App, ArgMatches};


fn main() {

    let mut is_arch = false;
    let mut is_debug = false;
    let mut is_prog  = false;
    let mut is_res = false;

    let mut res_file = String::from("");
    let mut hash_file = String::from("");
    let mut work_dir = String::from("");
    let mut in_size = String::from("0 b");

    let mut jobs = 1;

    let mut exts = "*";

    let yaml = load_yaml!("../dupe_args.yml");

    let matches = App::from(yaml).get_matches();

    let paths = matches.value_of("dir").unwrap();
    let paths: Vec<String> =  paths.split(",").map(|s| s.to_string()).collect();

    // Deal with our flag options
    if matches.is_present("archive") {
        is_arch = true;
    }

    if matches.is_present("debug") {
        is_debug = true;
    }

    if matches.is_present("prog") {
        is_prog = true;
    }

    //
    if let Some(in_size) = matches.value_of("size") {
        println!("Value for input_size: {}", in_size);
    }

    if let Some(res_f) = matches.value_of("resume") {
        res_file = res_f.to_string();
        is_res = true;
    }

    if let Some(hash_f) = matches.value_of("hash") {
        hash_file = hash_f.to_string();
    }

    if let Some(work_d) = matches.value_of("work_dir") {
        work_dir = work_d.to_string();
    }

    if let Some(n_jobs) = matches.value_of("jobs") {
        match n_jobs.parse::<i32>() {
            Ok(n) => jobs = n,
            Err(e) => {
                println!("Number of jobs specificed, {}, is not a valid number!", n_jobs);
                process::exit(1);

            },
        }
    }
//    let conf = proc_in(matches);

    let conf = Config { search_path: paths, archive : is_arch, debug : is_debug, prog: is_prog,
        resume : is_res, res_file : res_file, size : in_size, jobs: jobs,
        work_file : work_dir, hash_file : hash_file, exts : exts };

    println!("conf: {:?}", conf);
}


//fn proc_in(matches: ArgMatches) -> Config {
fn proc_in(matches: ArgMatches) {
    // If we have a dir (we have to have a dir, otherwise clap would
    // have error'd out already
    if let Some(d) = matches.value_of("dir") {
        let path_vec: Vec<&str> = d.split(",").collect();
    }

    //let conf = Config::new();

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
/*
impl Config {
    pub fn new(yams: &Yaml) {
        let matches = App::from(yams).get_matches();

        let paths = matches.value_of("dir").unwrap();
        let path_vec: Vec<&str> = d.split(",").collect();


        // Deal with our flag options
        if matches.is_present("archive") {
            is_arch = true;
        } else {
            is_arch = false;
        }

        if matches.is_present("debug") {
            is_debug = true;
        } else {
            is_debug = false;
        }

        if matches.is_present("prog") {
            is_prog = true;
        } else {
            is_prog = false;
        }

        //
        if let Some(in_size) = matches.value_of("size") {
        } else {
            in_size = "0 B"
        }



        println!("Awesomeness is turned on");
        }

        Config { dir: path_vec, arch : is_arch, debug : is_debug, prog: is_prog,
                 }
        dir: Vec<String>, arch: bool, debug: bool, prog: bool,
               res: bool, res_file: String, size: String, jobs: u8,
               work : String, hash: String, exts: Vec<String>) -> Result<Config, &'static str> {



        Ok(Config { query: q, file })


    }
}*/
