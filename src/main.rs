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

    for t in rx.iter() {
        //let (sha, path) = t.unwrap();
        println!("{:?}", t);
    }
}


/*    if ((conf.exts.len() > 1) || (curr_exts[0] == "*")) {
        for x in conf.exts.iter() {
            walker.
        }
    }*/

 /*   let par = jwalk::Parallelism::Serial;
    if conf.jobs > 1 {
        let par = jwalk::Parallelism::RayonNewPool(conf.jobs.into());
    }

    let curr_dir = &conf.search_path[0];
    let walker = jwalk::WalkDirGeneric::<((usize),(bool))>::new(curr_dir)
        .min_depth(1)
        .parallelism(par);

    walker.process_read_dir(|depth, path, read_dir_state, children| {
                children.retain(|dir_entry_result| {
                    let curr_path = dir_entry_result.path().as_path();
                    let curr_meta = dir_entry_result.metadata().unwrap();
                    let fs = curr_meta.len();
                    (is_good_ext(curr_path, &conf.exts)) &&
                        (is_good_size(fs, conf.size))
                });
            });


    for entry in walker {
        println!("{}", entry?.path().display());
    }


*/
    // Borrowing a lot from:
    //     https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#calculate-sha256-sum-of-iso-files-concurrently
    //let pool = ThreadPool::new(conf.jobs.into());

    //let (tx, rx) = channel();

// Create a channel of unbounded capacity.
/*    let (s, r) = crossbeam_channel::unbounded();

    for x in conf.search_path.iter() {
        s.send(Path::new(x));
    }

    while pool.active_count() > 0 {
        pool.execute(move || {
            let curr_dir = r.recv().unwrap();

            // Read the contents of this directory and take appropriate action
            let contents = curr_dir.read_dir().unwrap();

            for x in contents.into_iter() {
                let curr_dir_entry = x.unwrap().clone();
                let curr_path = curr_dir_entry.path().to_owned().as_path();
                let curr_meta = curr_dir_entry.metadata().unwrap();

                if curr_meta.is_dir() {
                    s.send(curr_path);

                // Otherwise it's a file!
                } else {
                    // Before pulling any info, let's make sure this file has extension we want
                    if is_good_ext(curr_path, &conf.exts) {

                        let path_str = String::from(curr_path.to_str().unwrap());
                        let fs = curr_meta.len();

                        if fs >= conf.size {
                            file_res.push(FileResult::new(path_str, fs));
                        }
                    }
                }
            }
            });
    }*/
       /* for x in conf.search_path.iter() {
            for entry in WalkDir::new(x)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| ((!e.path().is_dir()) &&
                                     (is_good_ext(e.path(),&conf.exts)))) {
            let path = entry.path().to_owned();
            //let tx = tx.clone();
            pool.execute(move || {
                let fs = path.metadata().unwrap().len();

                let path_str = String::from(path.to_str().unwrap());

                // Only keep track if its within our size requirement.
                if fs >= conf.size {
                    file_res.push(FileResult::new(path_str, fs));
                }

                //tx.send((path_str, fs)).expect("Could not send data!");
            });
        }
    }
*/


    //drop(tx);
/*    for t in file_res.iter() {
        println!("{:?} ", t);
    }*/






 /*   let walker = WalkDir::new(curr_dir);

    for entry in walker {
        let curr_en = match entry {
            Ok(f) => f,
            Err(e) => {
                println!("DirEntry Error: {}", e);
                continue
            },
        };

        let curr_path  =  curr_en.path();

/*        let curr_fn = match    curr_path.file_name() {
            Ok(f) => f,
            Err(e) => {
                println!("Path Error: [{}] {:?}", e, curr_en);
                continue
            }
        };*/

/*        let canon_path  = match curr_path.canonicalize() {
            Ok(f) => f,
            Err(e) => {
                println!("Canonicalize Error: [{:?}] {:?}", e, curr_en);
                continue
            }
        };*/

        if curr_path.is_dir() {
            continue
        }

        let ext_check = is_good_ext(curr_path, &conf.exts);

        let fs = match curr_path.metadata() {
            Ok(f) => f.len(),
            Err(e) => {
                println!("Metadata Error! [{:?}] {:?}", e, curr_path);
                continue
            }
        };

        if ext_check {
            //println!("[{}] {}", fs, curr_path.to_str().unwrap())
        }



    }*/





  /*  // Load in our YAML file describing program options and extract user input

    let par = jwalk::Parallelism::Serial;
    if conf.jobs > 1 {
        let par = jwalk::Parallelism::RayonNewPool(conf.jobs.into());
    }*/
/*
    let walker = jwalk::WalkDirGeneric::<((usize),(bool))>::new(curr_dir)
        .min_depth(1)
        .parallelism(par)
        .process_read_dir(|read_dir_state, children| {
            children.retain(|dir_entry_result| {
                dir_entry_result.as_ref().map(|dir_entry| {
                    dir_entry.file_name
                        .to_str()
                        .map(|s| s.ends_with(".txt"))
                        .unwrap_or(true)
                }).unwrap_or(true)
            });
        });*/


/*    let walker = WalkDir::new(curr_dir)
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


    } */

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