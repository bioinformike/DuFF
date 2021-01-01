mod util;
mod config;
mod file_result;

use crate::util::*;
use crate::config::*;
use crate::file_result::*;

use std::{process, fmt, env, path::Path};
use clap::{load_yaml, App, ArgMatches};



use crossbeam::crossbeam_channel;

use walkdir::WalkDir;
use ignore::WalkBuilder;


use ring::digest::{Context, Digest, SHA256};


fn main() {

    // Get user input
    let yams = load_yaml!("../dupe_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);

    // Try creating our files and if we can't tell the user that we don't have the write
    // permissions we need for either the directory they specified or cwd
    let mut work_file = open_file(&conf.work_file, &conf.work_dir,
                                        conf.user_set_dir);

    let mut hash_file = open_file(&conf.hash_file, &conf.work_dir,
                                  conf.user_set_dir);

    let mut log_file = open_file(&conf.log_file, &conf.work_dir,
                                  conf.user_set_dir);

    let mut temp_file = open_file(&conf.temp_file, &conf.work_dir,
                                  conf.user_set_dir);

    let mut report_file = open_file(&conf.report_file, &conf.work_dir,
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



