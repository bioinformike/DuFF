mod util;
mod config;
mod file_result;

use crate::util::*;
use crate::config::*;
use crate::file_result::*;

use std::{path::Path, fs::File, io};
use clap::{load_yaml, App};

use crossbeam::crossbeam_channel;
use walkdir;
use ignore;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use std::process::exit;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator};
use indicatif::{ParallelProgressIterator, ProgressStyle, ProgressBar, ProgressDrawTarget};

use console::{Emoji, style};

static LOOKING_GLASS: Emoji = Emoji("🔍", "");
static ROCKET: Emoji = Emoji("🚀", "");
static SCALES: Emoji = Emoji("🧭", "");
static TREE: Emoji = Emoji("🌴", "");
static ROBOT: Emoji = Emoji("🤖", "");
static MONOCLE: Emoji = Emoji("🧐", "");
static CLAPPER: Emoji = Emoji("🎬", "");
static REPORT: Emoji = Emoji("📃️", "");

fn main() {


    // Get user input
    let yams = load_yaml!("../duff_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = Config::new(matches);


    if conf.prog {
        println!("[{}, {}] {} Initializing DuFF...",
                 dt(),
                 style("01/11").bold().dim(),
                 ROCKET
        );
    }
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

    let file_res: Vec<FileResult> = vec![];

    let (tx, rx) = crossbeam_channel::unbounded::<FileResult>();

    let mut dirs = conf.search_path.clone();
    let curr_dir = dirs.pop().unwrap();

    let mut walker = ignore::WalkBuilder::new(Path::new(curr_dir.as_str()));

    for x in dirs.iter() {
        walker.add(Path::new(x.as_str()));
    }
    walker.threads(conf.jobs as usize);
    let walker = walker.build_parallel();


    if conf.prog {
        let spin = ProgressBar::new_spinner();
        spin.set_draw_target(ProgressDrawTarget::stdout());
        spin.enable_steady_tick(120);
        spin.set_style(
            ProgressStyle::default_spinner()
                // For more spinners check out the cli-spinners project:
                // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                .tick_strings(&[
                    "▰▱▱▱▱▱▱",
                    "▰▰▱▱▱▱▱",
                    "▰▰▰▱▱▱▱",
                    "▰▰▰▰▱▱▱",
                    "▰▰▰▰▰▱▱",
                    "▰▰▰▰▰▰▱",
                    "▰▰▰▰▰▰▰",
                ])
                .template("{prefix} {spinner:.blue}"),
        );
        spin.set_prefix(&format!("[{}, {}] {} Searching requested directories...",
                                 dt(),
                                 style("02/11").bold().dim(),
                                 LOOKING_GLASS));
    }
    walker.run( || {
        spin.inc(1);
        let tx = tx.clone();
        let conf = conf.clone();
        Box::new(move |result| {
            use ignore::WalkState::*;
            let curr_dir = match result {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[Extract curr_dir error] {}", e);
                    return ignore::WalkState::Continue;
                }
            };

            let curr_path = curr_dir.path();

            // We don't care about directories!
            if curr_path.is_dir() {
                return ignore::WalkState::Continue;
            }

            let path_str = match curr_path.to_str() {
                Some(t) => t,
                None => {
                    eprintln!("Error path-> path_str");
                    return ignore::WalkState::Continue;
                }
            };

            let path_str = String::from(path_str);

            let curr_meta = match curr_dir.metadata() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[Meta error] {}", e);
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

    if conf.prog {
        println!("[{}, {}] {} Building file size tree...",
                 dt(),
                 style("03/11").bold().dim(),
                 TREE
        );
    }

    // Dump the channel contents out into a vec
    for t in rx.iter() {
        // Thanks to this SO answer: https://stackoverflow.com/a/33243862
        dict.entry(t.size).or_insert(Vec::new()).push(t);
    }

    drop(rx);

    if conf.prog {
        println!("[{}, {}] {} Identifying duplicate files by size...",
                 dt(),
                 style("04/11").bold().dim(),
                 SCALES
        );
    }

    dict.retain(|&k, v| v.len() > 1);

    if dict.len() == 0 {
        println!("No duplicate files!");
        exit(0)
    }

    if conf.prog {
        println!("[{}, {}] {} Found {} duplicate files by size...",
                 dt(),
                 style("05/11").bold().dim(),
                 MONOCLE,
                 dict.len()
        );
    }

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

    if conf.prog {
        println!("[{}, {}] {} Calculating hashes for all duplicate files...",
                 dt(),
                 style("06/11").bold().dim(),
                 ROBOT,
        );
    }

    if conf.prog {
        let pb = ProgressBar::with_draw_target(flat.len() as u64,
                                               ProgressDrawTarget::stdout());

        pb.set_style(ProgressStyle::default_bar()
            .template("[{percent}%, {elapsed_precise}] [{bar:60.cyan/blue}] {pos:>7}/{len:7} ({eta})")
            .progress_chars("#>-"));

        flat.par_iter_mut().progress_with(pb).for_each(|x| {
            let start = Instant::now();
            x.calc_hash(8192);
            tx.send(x.to_owned()).unwrap();

            let duration = start.elapsed();
            //println ! ("{:?}, {:?}, {}, {},  {}", duration, 8192, & x.hash, &x.size, & x.file_path);
        });
    } else
    {
        flat.par_iter_mut().for_each(|x| {
            let start = Instant::now();
            x.calc_hash(8192);
            tx.send(x.to_owned()).unwrap();

            let duration = start.elapsed();
        });
    }

    // Slide modification of what we did above after the directory walking
    let mut dict = HashMap::new();

    drop(tx);

    if conf.prog {
        println!("[{}, {}] {} Building hash tree...",
                 dt(),
                 style("07/11").bold().dim(),
                 TREE
        );
    }

    // Dump the channel contents out into a vec
    for t in rx.iter() {
        let key = format!("{}_{}", t.size, t.hash);

        dict.entry(key).or_insert(Vec::new()).push(t);
    }

    drop(rx);

    if conf.prog {
        println!("[{}, {}] {} Identifying duplicate files by hash...",
                 dt(),
                 style("08/11").bold().dim(),
                 SCALES
        );
    }

    dict.retain(|_, v| v.len() > 1);

    if dict.len() == 0 {
        println!("No duplicate files!");
        exit(0)
    }

    if conf.prog {
        println!("[{}, {}] {} Found {} duplicate files.",
                 dt(),
                 style("09/11").bold().dim(),
                 MONOCLE,
                 dict.len()
        );
    }

    if conf.prog {
        println!("[{}, {}] {} Wrapping up...",
                 dt(),
                 style("10/11").bold().dim(),
                 CLAPPER,
        );
    }

    if conf.prog {
        println!("[{}, {}] {} Writing report...",
                 dt(),
                 style("11/11").bold().dim(),
                 REPORT,
        );
    }


}



