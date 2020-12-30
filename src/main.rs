use std::process;
use clap::{load_yaml, App, ArgMatches};
use walkdir::WalkDir;
use globwalk::GlobWalkerBuilder;
use std::fs::FileType;
use std::fs;
use jwalk::WalkDirGeneric;
use rayon::ThreadPoolBuilder;
use std::{thread, time};

use crossbeam::crossbeam_channel::unbounded;
use std::path::Path;

use std::fs::File;
use std::io::{BufReader, Read, Error};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use ring::digest::{Context, Digest, SHA256};

/*fn is_good_ext(curr_path: Path) -> bool {
    if path.
}*/

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

fn main()   {

    let pool = ThreadPool::new(5);

    let yams = load_yaml!("../dupe_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);

    println!("conf: {:?}", conf);

    let curr_dir = &conf.search_path[0];

    println!("Searching {}", curr_dir);

    let walker = WalkDir::new(curr_dir);

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



    }

    /*for entry in WalkDir::new(curr_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().is_dir() && is_good_ext(e.path())) {
                println!("{:?}", entry);
        }
        //let (tx, rx) = channel();


*/


/*
    let path = entry.path().to_owned();
    let tx = tx.clone();
    pool.execute(move || {
        let digest = compute_digest(path);
        tx.send(digest).expect("Could not send data!");
    });*/
/*    drop(tx);
    for t in rx.iter() {
        let (sha, path) = t?;
        println!("{:?} {:?}", sha, path);
    }
    Ok(())*/
}


fn compute_digest<P: AsRef<Path>>(filepath: P) -> Result<(Digest, P), Error> {
    let mut buf_reader = BufReader::new(File::open(&filepath)?);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = buf_reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok((context.finish(), filepath))
}

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
    size     : String,
    jobs     : u8,

    work_file : String,
    hash_file : String,
    exts      : Vec<String>
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
            println!("Value for input_size: {}", in_size);
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