mod util;
mod config;
mod file_result;

use crate::util::*;
use crate::config::*;
use crate::file_result::*;

use std::{path::Path, fs::File, io};
use clap::{load_yaml, App};

use crossbeam::crossbeam_channel;
//use itertools::Itertools;
use walkdir;
use ignore;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use std::process::exit;
use rayon::prelude::*;



fn main() {

    // Get user input
    let yams = load_yaml!("../duff_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);

    // Try creating our files and if we can't tell the user that we don't have the write
    // permissions we need for either the directory they specified or cwd
    let work_file = open_file(&conf.work_file, &conf.work_dir,
                                        conf.user_set_dir);

    let hash_file = open_file(&conf.hash_file, &conf.work_dir,
                                  conf.user_set_dir);

    let log_file = open_file(&conf.log_file, &conf.work_dir,
                                  conf.user_set_dir);

    let temp_file = open_file(&conf.temp_file, &conf.work_dir,
                                  conf.user_set_dir);

    let report_file = open_file(&conf.report_file, &conf.work_dir,
                                  conf.user_set_dir);


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

            let mtime = curr_meta.modified().unwrap();



            let fs = u128::from(curr_meta.len());

            // only want to send something down the channel if its a file and meets our extension
            // and size requirements.
            let ext_match = is_good_ext(curr_path, &conf.exts);
            let size_match = is_good_size(fs, conf.ll_size, conf.ul_size);
            if ext_match && size_match  {
                tx.send(FileResult::new(path_str, fs, mtime)).unwrap();
            }

            Continue
        })
    });

    drop(tx);

    let mut dict = HashMap::new();

    // Dump the channel contents out into a vec
    for t in rx.iter() {
        // Thanks to this SO answer: https://stackoverflow.com/a/33243862
        dict.entry(t.size).or_insert(Vec::new()).push(t);
    }

    drop(rx);

    dict.retain(|&k, v| v.len() > 1);

    if dict.len() == 0 {
        println!("No duplicate files!");
        exit(0)
    }

    // Loop to push all of our FileResult structs to do hash calculation in parallel
/*    for (k, v) in dict.drain() {
        for x in v.drain(0..) {
            // push to pool of threads to consume and:
            // 1. Calculate hash for the file represented by FileResult
            // 2. Update the FileResult object with the hash
            // 3. Push the FileResult object down a channel for further processing.
        }
    }*/


    let (tx, rx) = crossbeam_channel::unbounded::<FileResult>();

    let flat: Vec<_> = dict.values().collect();
    let mut flat: Vec<_> = flat.into_iter().flatten().cloned().collect::<Vec<_>>();

/*//                        1 KiB  8 KiB 128KiB 256KiB  512KiB  1MiB, 8MiB, 16MiB, 32MiB]
    let mut sizes = [1024, 8192, 131072, 262144, 524288, 1048576, 8388608,
                              16777216, 33554432];
        for z in 0..sizes.len() {
            flat.par_iter_mut().for_each( |x| {

                let start = Instant::now();
                x.calc_hash(sizes[z]);
                tx.send(x.to_owned()).unwrap();

                let duration = start.elapsed();
                println ! ("{:?}, {:?}, {}, {},  {}", duration, sizes[z], & x.hash, &x.size, & x.file_path);
            });
    }*/

    flat.par_iter_mut().for_each( |x| {

        let start = Instant::now();
        x.calc_hash(8192);
        tx.send(x.to_owned()).unwrap();

        let duration = start.elapsed();
        println ! ("{:?}, {:?}, {}, {},  {}", duration, 8192, & x.hash, &x.size, & x.file_path);
    });

    // Slide modification of what we did above after the directory walking
    let mut dict = HashMap::new();

    drop(tx);

    // Dump the channel contents out into a vec
    for t in rx.iter() {
        let key = format!("{}_{}", t.size, t.hash);

        dict.entry(key).or_insert(Vec::new()).push(t);
    }

    //drop(rx);

    dict.retain(|k, v| v.len() > 1);

    if dict.len() == 0 {
        println!("No duplicate files!");
        exit(0)
    }


}



