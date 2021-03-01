mod util;
mod config;
mod file_result;

// To use our wrapper function for creating files for writing to.
use self::util::open_file;

// To hold the configuration details for this DuFF run.
use self::config::Config;


// Standard library stuff:
// For file paths and such
use std::path::{Path,PathBuf};

// For writing to our output files.
use std::io::Write;

// For deduplicating we use a hashmap struct to make it a bit easier.
use std::collections::HashMap;

// To quit early if errors are detected that cannot be dealt with.
use std::process::exit;


// Parallelism crates:
// For directory traversal work.
use crossbeam_deque::{Injector, Worker};

// For file examination and hash calculation
use crossbeam::{crossbeam_channel, thread};

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

// For calculating mtime
use chrono::Utc;


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
    let yams = load_yaml!("../duff_args.yml");
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

    // Open the report file for writing
    let mut report_file = open_file(&conf.report_file, &conf.out_dir,
                                    conf.user_set_dir);

    // Open the log file for writing.
    // TODO: This should only be done if the user requested a log file.
    let mut log_file = open_file(&conf.log_file, &conf.out_dir,
                                    conf.user_set_dir);

    if !conf.silent {
        println!("{}", conf)
    }

    // TODO: This should only be run if the user requested a log file.
    // Write out the configuration to the log file
    writeln!(log_file, "{}", conf);


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
    let mut spin = ProgressBar::new_spinner();

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
        spin.set_prefix(&format!("[{}, {}] {} Traversing directories...",
                                 util::dt(),
                                 style("02/11").bold().dim(),
                                 FILES));
    }



    // Traversing directories in a multi-threaded fashion, pushing any directories we run into on
    // global work queue (global_q) and throwing any files we find down tx channel, for further
    // processing.
    thread::scope(|scope| {

        // Only spool up as many jobs as the user request (defaults to 1).
        for x in 0..conf.jobs {

            // Clone our channel for each thread.
            let tx = tx.clone();
            scope.spawn(move |_| {

                // Create this threads local work queue
                let mut local_q: Worker<PathBuf> = Worker::new_fifo();

                // Start traversing those directories, grabbing another from the global queue when
                // finished looking at the current one!
                loop {
                    match util::find_task(&mut local_q, &global_q) {
                        Some(mut job) => {

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
                                                tx.send(curr_pb);
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
            Some(mut fr) => {
                tx.send(fr);
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
    dict.retain(|&k, v| v.len() > 1);

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

    // Iterate through all FileResult structs in flat using the calc_hash function
    // to calculate a hash, and shove the updated FileResult struct down tx. NB the calc_hash
    // function updates the internal struct hash value.
    flat.par_iter_mut().for_each(|x| {

        pb.inc(1);

        x.calc_hash(buff_size);

        tx.send(x.to_owned()).unwrap();

    });

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
        exit(0)
    }

    // Let the user know how many duplicate files we found.
    if !conf.hide_prog {
        println!("[{}, {}] {} Found {} files that have been duplicated, totaling {} duplicate \
                 files.",
                 util::dt(),
                 style("09/11").bold().dim(),
                 MONOCLE,
                 n_uniq,
                 n_dupes
        );
    }

    // TODO: Do we actually need this, or can we just skip to writing report?
    if !conf.hide_prog {
        println!("[{}, {}] {} Wrapping up...",
                 util::dt(),
                 style("10/11").bold().dim(),
                 CLAPPER,
        );
    }

    // Letting the user know we are writing the report and where they can find it again.
    if !conf.hide_prog {
        println!("[{}, {}] {} Writing report [{}]...",
                 util::dt(),
                 style("11/11").bold().dim(),
                 REPORT,
                 conf.report_file
        );
    }

    // TODO: Do we need some sort of progress indicator here? How long could this take??
    // TODO: This format is more for the hash/resume file, we need a better reporting format.
    // Write the header line
    writeln!(report_file, "size hash mtime path");
    for (k, v) in dict.iter() {
        for y in v.iter() {
            writeln!(report_file, "{}", y);
        }
    }
}



