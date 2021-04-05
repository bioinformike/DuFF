// I sometimes use extra parens to make thing more readable to me
#![allow(unused_parens)]

mod util;
mod config;
mod file_result;

// To use our wrapper function for creating files for writing to.
use self::util::open_file;

// Standard library stuff:
// For file paths and such
use std::path::PathBuf;

// For writing to our output files.
use std::io::{Write, BufReader, BufRead};

// For deduplicating we use a hashmap struct to make it a bit easier.
use std::collections::HashMap;

// To quit early if errors are detected that cannot be dealt with.
use std::process::exit;


// Parallelism crates:
// For directory traversal work.
use crossbeam_deque::{Injector, Worker};

// For file examination and hash calculation
use crossbeam_channel;
use crossbeam_utils::thread;

// For use of par_iter for processing files and calculating hashes.
use rayon::iter::{ParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator};


// Progress crates:
// For displaying progress bars and spinners to the user
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};

// To update the user with helpful status messages.
use console::{Emoji, style};

// Miscellaneous crates
// For dealing with command line arguments
use clap::{load_yaml, App};
use std::fs::File;
use shh;


// Different emojis that we use to show indicate what the program is doing.
static LOOKING_GLASS: Emoji = Emoji("🔍", "");
static FILES: Emoji = Emoji("🗃️", "");
static ROCKET: Emoji = Emoji("🚀", "");
static SCALES: Emoji = Emoji("🧭", "");
static TREE: Emoji = Emoji("🌴", "");
static ROBOT: Emoji = Emoji("🤖", "");
static MONOCLE: Emoji = Emoji("🧐", "");
static CLAPPER: Emoji = Emoji("🎬", "");
static REPORT: Emoji = Emoji("📃️", "");

fn main() {




    // Get user input
    let yams = load_yaml!("duff_args.yml");
    let matches = App::from(yams).get_matches();

    // Process user input
    let conf = config::Config::new(matches);

    // Give the user some immediate feedback that DuFF is running.
    if !conf.hide_prog {
        println!("[{}, {}] {} Initializing DuFF...",
                 util::dt(),
                 style("01/11").bold().dim(),
                 ROCKET
        );
    }

    // Setup shh to drop all stderr warnings if that's what the user wants
    let shh = shh::stderr().unwrap();

    if !conf.hide_err {
        drop(shh);
    }

    // Setup the rayon threadpool which we will use later
    rayon::ThreadPoolBuilder::new().num_threads(conf.jobs as usize).build_global().unwrap();

    // Open the report file for writing
    let report_file = open_file(&conf.report_file, &conf.out_dir,
                                    conf.user_set_dir);

    // Open the log file for writing - this file is hidden if the user didn't want it and will be
    // cleaned up.
    let mut log_file = open_file(&conf.log_file, &conf.out_dir,
                                     conf.user_set_dir);


    // Open the archive file for writing - this file is hidden if the user didn't want it and will
    // be cleaned up.
    let arch_file = open_file(&conf.archive_file, &conf.out_dir,
                                 conf.user_set_dir);

    // Create dictionary for hashes from previous run.
    let mut prev_dict: HashMap<u128, Vec<file_result::FileResult>> = HashMap::new();


    if !conf.silent {
        println!("{}", conf)
    }

    // Write out the configuration to the log file
    if conf.log {

        // TODO: Replace unwrap
        writeln!(log_file, "#Config\n{}\n#Starting file search\n", conf).unwrap();
    }


    // Logic to handle a resuming of previous DuFF run using log file from that run
    if conf.resume {

    }

    // Logic to handle hash file from previous DuFF run
    if conf.have_hash {
        let prev_hash_file = match File::open(&conf.prev_hash_file) {
            Ok(t) => t,
            Err(e) => {

                // If we can't read the file kill the program
                eprintln!("[Error Reading previous hash file] {}", e);
                println!("Error reading input previous hash file {}. \nPlease fix and try to re-run \
                        or re-run without using this file.", &conf.prev_hash_file);
                exit(1)

            }
        };

        let hash_reader = BufReader::new(prev_hash_file);

        for line in hash_reader.lines() {
            let curr_line = match line {
                Ok(t) => t,
                Err(_) => continue
            };

            let curr_obj: file_result::FileResult = match serde_json::from_str(&curr_line) {
                Ok(t) => t,

                Err(_) => {
                    // Error reading line, just skip to next
                    continue

                }
            };

            //let fs: u128 = t.size;
            // Thanks to this SO answer: https://stackoverflow.com/a/33243862
            prev_dict.entry(curr_obj.size).or_insert(Vec::new()).push(curr_obj);
        }

    }


    // Create our channels that we will use to send the files we find during directory traversal
    // down for further processing later.
    let (tx, rx) =
        crossbeam_channel::unbounded::<PathBuf>();


    // Nifty trick picked up from Ken Sternberg's parallel Boggle Solver
    // [https://github.com/elfsternberg/boggle-solver/blob/4dbb9b9e07da493c74fe9299fa8fb7d5b5589151/docs/20190816_Solving_Boggle_Multithreaded.md]
    let global_q = &{
        let global_q = Injector::new();

        // Push our initial directories to search given to use by the user.
        for x in conf.search_path.iter() {
            global_q.push(PathBuf::from(x))
        }

        global_q
    };

    // Setup a spinner to let the user know we are traversing the directory structure.  This didn't
    // seem like a place to use an actual progress bar since we don't know how many directories we
    // will actually be traversing.  We could have made it just the length of the user provided
    // input directories, but from my personal experience with my use case this will be less than
    // informative as each directory I give it could take quite a long time to actually traverse.
    // Initialize the spinner out here, so the compiler won't yell at me.
    let spin = ProgressBar::new_spinner();

    // Update some things about the spinner and get it spinning using steady tick.  We will manually
    // end the ticking after the traversal is complete.
    if !conf.hide_prog {
        spin.set_draw_target(ProgressDrawTarget::stdout());
        spin.enable_steady_tick(120);
        spin.set_style(
            ProgressStyle::default_spinner()
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
        spin.set_prefix(&format!("[{}, {}] {}  Traversing directories...",
                                 util::dt(),
                                 style("02/11").bold().dim(),
                                 FILES));
    }



    // Traversing directories in a multi-threaded fashion, pushing any directories we run into on
    // global work queue (global_q) and throwing any files we find down tx channel, for further
    // processing.
    thread::scope(|scope| {

        // Only spool up as many jobs as the user request (defaults to 1).
        for _x in 0..conf.jobs {

            // Clone our channel for each thread.
            let tx = tx.clone();
            scope.spawn(move |_| {

                // Create this threads local work queue
                let mut local_q: Worker<PathBuf> = Worker::new_fifo();

                // Start traversing those directories, grabbing another from the global queue when
                // finished looking at the current one!
                loop {
                    match util::find_task(&mut local_q, &global_q) {
                        Some(job) => {

                            // This should be the only case - as only directories will be pushed
                            // into the global queue and then pulled down into a thread's local
                            // queue.
                            if job.is_dir() {

                                // Read contents of dir
                                let dir_ls = match job.read_dir() {
                                    Ok(t) => t,
                                    Err(e) => {
                                        eprintln!("[Readdir error] {}", e);
                                        continue
                                    }
                                };

                                // Process the contents of the directory
                                for entry in dir_ls {
                                    match entry {

                                        // There might be a better way to handle this.
                                        Ok(curr_ent) => {

                                            // Grab the path from the DirEntry as a PathBuf which
                                            // we will send down the channel.
                                            let curr_pb = curr_ent.path();

                                            // Get Path version of our PathBuf so we can check if
                                            // it's a directory.
                                            let curr_path = curr_pb.as_path();

                                            // If the Path is a directory push it into global q.
                                            // Otherwise, assume it's a file and send it down the
                                            // channel.
                                            if curr_path.is_dir() {
                                                global_q.push(curr_pb);
                                            } else {
                                                tx.send(curr_pb).unwrap();
                                            }
                                        },

                                        // If there was an error with this DirEntry, not sure what
                                        // we can do beside let the user know and move on.
                                        Err(e) => {
                                            eprintln!("Error with DirEntry in dir {:?}: {}",job, e);
                                            continue
                                        }
                                    }
                                }
                            }
                        },
                        None => break,
                    }
                }
            });
        }
    }).unwrap();

    drop(tx);

    // Vector to hold all the PathBufs we found while traversing directories
    let mut f_ls: Vec<PathBuf> = Vec::new();

    // Dump the channel contents out into the vec
    for t in rx.iter() {
        f_ls.push(t);
    }

    drop(rx);

    // End the directory traversal spinner.
    spin.finish();

    // File processing progress bar, set to length of number of files found in directory traversal.
    // Initialized out here because if not the compiler complains.
    let mut pb = ProgressBar::new(f_ls.len() as u64);

    // Let the user know we have finished directory traversal and we are moving on to actually look
    // at all the files we found along the way.  Also, set up the progress bar some more.
    if !conf.hide_prog {

        println!("[{}, {}] {} Examining Files...",
                 util::dt(),
                 style("03/11").bold().dim(),
                 MONOCLE
        );

        pb.set_draw_target(ProgressDrawTarget::stdout());

        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:60}] {pos:>7}/{len:7} ({eta})"));

    }

    // Re-init as FileResult channels.
    let (tx, rx) =
        crossbeam_channel::unbounded::<file_result::FileResult>();

    // Process each file, getting it's file size and mtime, creating a FileResult object
    // and pushing that down tx.
    f_ls.par_iter().for_each( |x| {

        // Counting chickens...
        if !conf.hide_prog {
            pb.inc(1);
        }

        // Run our process_file function, which will give us back a FileResult struct wrapped in a
        // Some if this file, x, is able to be processed and matches the user's requested extension
        // and file size filters.
        match util::process_file(x, &conf) {
            Some(fr) => {

                // If the user wants the log, start logging the files
                if conf.log {
                    match serde_json::to_string(&fr) {
                        Ok(t) => {
                            writeln!(&log_file, "{}", t).unwrap();
                        }

                        Err(e) => {
                            eprintln!("[Serialization error] {}", e);
                        }
                    }


                }
                tx.send(fr).unwrap();
            }

            // Totally not sure if this is the way you are supposed to handle this, but it seems to
            // work...so... ¯\_(ツ)_/¯
            None => return
        }

    });

    // Finish off the file processing progress bar
    pb.finish();

    drop(tx);

    // Let the user know we are done with the file examination step and we are now building our
    // tree - which could maybe take some time?
    if !conf.hide_prog {
        println!("[{}, {}] {} Building initial file tree...",
                 util::dt(),
                 style("04/11").bold().dim(),
                 TREE
        );
    }


    // We will use a hashmap to collect our results because the key-value structure seemed to be a
    // natural choice when trying to find duplicate files by file size. Each key represents a
    // particular file size encountered and then the corresponding value is a vector of FileResult
    // structs, which will allow us to collate files by their file size making it easy to identify
    // potential duplicates (if the vector length is > 1).
    let mut dict = HashMap::new();

    // Dump the channel contents out into a vec
    for t in rx.iter() {
        // Thanks to this SO answer: https://stackoverflow.com/a/33243862
        dict.entry(t.size).or_insert(Vec::new()).push(t);
    }

    drop(rx);

    // Only keep an item in the hashmap if the key's (file size) corresponding value (vector of
    // FileResult structs) has at least 2 elements (dupes)
    dict.retain(|_, v| v.len() > 1);

    // Flatten the hashmap out in this annoying 2-step procedure for further processing. First we
    // dump all of the hashmap values (vectors of FileResult structs) into a single vector, and then
    // flatten that vector into 1 overall vector.
    let flat: Vec<_> = dict.values().collect();
    let mut flat: Vec<_> = flat.into_iter().flatten().cloned().collect::<Vec<_>>();

    // The number of duplicates we have should now be the length of our flattened tree's flattened
    // vectors.
    let n_dupes = flat.len() as u64;

    // TODO: We need to handle this better, writing out logs and reports if requested, instead of just quitting.
    if n_dupes == 0 {
        println!("No duplicate files!");
        util::clean_up(&conf);
        exit(0)
    }

    // Re-init the progress bar with the number of duplicate files.
    pb = ProgressBar::new(n_dupes);

    // Let the user know how many duplicate files we found (not n_uniq like below), then notify them
    // we are starting to calculate the hashes, while also setting the progress bar back up.
    if !conf.hide_prog {
        println!("[{}, {}] {} Found {} duplicate files by size...",
                 util::dt(),
                 style("05/11").bold().dim(),
                 LOOKING_GLASS,
                 n_dupes
        );


        println!("[{}, {}] {} Calculating File Hashes...",
                 util::dt(),
                 style("06/11").bold().dim(),
                 ROBOT
        );

        pb.set_draw_target(ProgressDrawTarget::stdout());

        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:60}] {pos:>7}/{len:7} ({eta})"));
    }

    // 512 KiB BufReader size
    let buff_size = 524288;

    // Re-init our FileResult channels
    let (tx, rx) =
        crossbeam_channel::unbounded::<file_result::FileResult>();

    if conf.log {

        // TODO: Replace unwrap
        writeln!(log_file, "#Starting hashing\n").unwrap();
    }

    // Iterate through all FileResult structs in flat using the calc_hash function
    // to calculate a hash, and shove the updated FileResult struct down tx. NB the calc_hash
    // function updates the internal struct hash value.
    flat.par_iter_mut().for_each(|x| {

        pb.inc(1);

        // Indicator to tell downstream code if we found a hash match for this file.
        let mut hash_match_found = false;

        // If we have a user input hash file we should check that first before hashing
        if conf.archive {

            match  prev_dict.get(&x.size) {
                Some(t) => {

                    for y in t {
                        let f_path = &y.file_path;
                        let f_hash = &y.hash;
                        let m_time = &y.mtime;

                        // If we have a file match (by size - key, path, and same mtime) grab it's hash
                        if ((f_path == &x.file_path) & (m_time == &x.mtime)) {
                            hash_match_found = true;
                            x.update_hash(f_hash.to_string());
                            break
                        }


                    }
                }
                None =>  (),
            };


        }

        // If we weren't able to find a match for this file then just calculate the hash as normal.
        if !hash_match_found {
            x.calc_hash(buff_size);
        }

        // If the user wants the log, start logging the files
        if conf.log | conf.archive {
            match serde_json::to_string(&x) {
                Ok(t) => {
                    if conf.log {
                        writeln!(&log_file, "{}", t).unwrap();
                    }
                    if conf.archive {
                        writeln!(&arch_file, "{}", t).unwrap();
                    }
                }

                Err(e) => {
                    eprintln!("[Serialization error] {}", e);
                }
            }
        }


        tx.send(x.to_owned()).unwrap();

    });

    // Finish off the file processing progress bar
    pb.finish();

    // Slight modification of what we did above after the directory walking
    let mut dict = HashMap::new();

    drop(tx);

    // Let them know we have finished the hash calculations and we are now building a "tree"
    if !conf.hide_prog {
        println!("[{}, {}] {} Building hash tree...",
                 util::dt(),
                 style("07/11").bold().dim(),
                 TREE
        );
    }

    // Dump all of the updated FileResult structs out of the channel and into a vec for further
    // processing.
    for t in rx.iter() {
        let key = format!("{}_{}", t.size, t.hash);

        dict.entry(key).or_insert(Vec::new()).push(t);
    }

    drop(rx);

    // Update the user to let them know we are about to 'collapse' the hashmap - this might take
    // some time? Need to do more testing to find out about that.
    if !conf.hide_prog {
        println!("[{}, {}] {} Identifying duplicate files by hash...",
                 util::dt(),
                 style("08/11").bold().dim(),
                 SCALES
        );
    }

    // Remove any entries from the hashmap that don't have at least 1 duplicate.
    dict.retain(|_, v| v.len() > 1);

    // n_dupes counts the total number of duplicate files, whereas n_uniq is the number of unique
    // files that have been duplicated.
    let mut n_dupes = 0;
    let mut n_uniq = 0;
    for (_, v) in dict.iter() {
        n_uniq += 1;
        n_dupes += v.len();
    }

    // TODO: Update this to still write out log files or whatever is needed even if no dupes
    if n_dupes == 0 {
        println!("No duplicate files!");
        util::clean_up(&conf);
        exit(0)
    }

    // Let the user know how many duplicate files we found.
    if !conf.hide_prog {
        println!("[{}, {}] {} Found {} unique files, and a total of {} duplicate \
                 files.",
                 util::dt(),
                 style("09/11").bold().dim(),
                 MONOCLE,
                 n_uniq,
                 n_dupes
        );
    }

    if !conf.hide_prog {
        println!("[{}, {}] {} Wrapping up...",
                 util::dt(),
                 style("10/11").bold().dim(),
                 CLAPPER,
        );
    }

    // If this function becomes more than just removing a log file if the user didn't request it,
    // we might need to move the location of this call.
    util::clean_up(&conf);


    // Letting the user know we are writing the report and where they can find it again.
    if !conf.hide_prog {
        println!("[{}, {}] {} Writing report [{}]...",
                 util::dt(),
                 style("11/11").bold().dim(),
                 REPORT,
                 conf.report_file
        );
    }

    util::write_report(report_file, dict);


}



