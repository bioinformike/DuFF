// For some constants and datetime functions
use crate::util;

// process: To exit when there are errors
// env: To grab the current working directory, if needed
// fs: To complete simple checks on input search directories.
// fmt: Display trait implementation
use std::{process, env, fs, fmt};

// Allows for reading in more human friendly values for lower and upper limits
use byte_unit::Byte;

// Allows for more human friendly printing of byte values for lower and upper limits.
use pretty_bytes::converter;

// clap makes all of this work, but more specificly needed here as Config::new takes an ArgMatches
// value as it's only argument.
use clap::ArgMatches;

// The Config struct is simply here to contain the user input that is obtained using clap.
#[derive(Debug, Clone)]
pub struct Config {

    // Required argument(s):
    // NB If the user only specifies a search_path we set out_dir to the current working directory,
    // and if we cannot write there then we fail, as we need to be at least able to write out the
    // report file.

    // The directories the user requested we search for duplicate files within. The user will give
    // DuFF a comma separated list of directories, but internally we simply store that as a vector
    // of Strings.
    pub search_path: Vec<String>,


    // Optional flags:

    // The archive flag tells DuFF to save the calculated hashes for future re-use.
    pub archive : bool,

    // The log flag tells DuFF to save a log file for future re-runs of DuFF where we can skip the
    // directory traversal and file examination, instead jumping directly to hashing.
    pub log : bool,

    // The hide_prog flag tells DuFF that the user doesn't want to see progress information.
    pub hide_prog : bool,

    // The silent flag tells DuFF that the user doesn't want anything put into stdout. This is
    // pretty similar to hide_prog, but with just hide_prog we will still echo the DuFF
    // configuration back to the user, but the silent flag disables this.
    pub silent : bool,


    // Optional Arguments:

    // ll_size corresponds to the user set lower limit file size, defaulting to 0 B. A file will be
    // required to be greater than or equal to the value of ll_size for further consideration. The
    // user input is read in and handled by byte_unit to convert human friendly values, i.e. 100 MB,
    // into actual bytes for DuFF internal use.
    pub ll_size : u128,

    // ul_size holds the user specified upper limit file size, defaulting to the max value of a u128
    // 340282366920938463463374607431768211455 B or approx 340 Yottabytes. A file will be required
    // to be less than or equal to the value of ul_size to be considered further. User input for
    // ul_size is handled in the same manner as for ll_size.
    pub ul_size : u128,

    // jobs will hold the number of "jobs" or threads the user wants us to run DuFF with, defaulting
    // to 1 thread.
    pub jobs : u64,

    // exts holds the extensions that the user wanted to limit our search of files to.  The user
    // inputs a comma separated list of extensions on the command line, but we parse that and
    // internally store each extension as a String in this vector.  If the user doesn't specify any
    // extensions then a single String will be added to this vector, "*", telling the code there is
    // no user requested extension filtering to worry about.
    pub exts : Vec<String>,

    // out_dir will hold the directory the user wants us to write files to, defaulting to the
    // current working directory.  If we cannot write to out_dir, the program will fail, letting the
    // user know the reason.
    pub out_dir : String,

    // res_file will hold a path to a user provided log file produced from a previous DuFF run with
    // the log flag on.
    pub res_file : String,

    // prev_hash_file will hold the path to where the user specified 'hash file' is located, which
    // is the file that gets generated running DuFF with the archive flag on.
    pub prev_hash_file : String,


    // INTERNAL ARGUMENTS: Arguments not directly set by the user, but set in response to different
    //                     user input.

    // A string representing the path of where hashes should be saved if the user requested them to
    // be saved with the archive flag. It will be inside the specified or default out_dir.
    pub archive_file : String,

    // A string representing the path of where the log should be saved if the user requested it be
    // saved with the log flag. It will be inside the specified or default out_dir.
    pub log_file : String,

    // A string representing the path to where the report will be written. It will be inside the
    // specified or default out_dir.
    pub report_file: String,


    // INTERNAL FLAGS: Flags not directly set by the user, but set in response to different user
    //                 input.

    // The resume flag is set by DuFF if the user provides a previous DuFF log generated
    // using the debug flag.
    pub resume : bool,

    // The have_hash flag is set by DuFF if the user provides a hash file (using -hash argument).
    // This hash file must be generated by a previous run of DuFF in which the user specified the -a
    // flag.
    pub have_hash : bool,

    // The user_set_dir flag is set by DuFF if the user provides an output directory (using the -out
    // argument).  This flag is used to decide which error messages should be sent to the user if
    // we try to open the report file for writing, but it is determine we cannot for some reason.
    pub user_set_dir : bool,
}


impl Config  {
    pub fn new(in_args: ArgMatches) -> Config {

        // Initialize a bunch of placeholders that we will use to generate a new Config struct.
        // Required argument(s):
        // We don't initialize any required parameters up here, because since they are required, if
        // the user didn't specify them, the program would have already failed.

        // Optional flags:
        let mut archive = false;
        let mut log = false;
        let mut hide_prog = false;
        let mut silent = false;

        // Optional Arguments:

        // Default lower limit is 0 B. File must be >= this value.
        let mut ll_size = 0;

        // Default upper limit is 340282366920938463463374607431768211455 B or approx 340
        // Yottabytes, sooo we should not- I'm sure I'll regret this for some reason. File must be
        // <= this upper limit.
        let mut ul_size = 340282366920938463463374607431768211455;

        // Default number of threads is 1.
        let mut jobs = 1;

        // Default extension is just "*".
        let mut exts: Vec<String> = vec![String::from("*")];

        // out_dir needs to be mentioned up here for the compiler to be happy.
        let mut out_dir;

        // Files from previous work - initialize to empty strings, we use bools to tell DuFF whether
        // to try to open these for writing.
        let mut res_file = String::from("");
        let mut prev_hash_file = String::from("");

        // INTERNAL ARGUMENTS:
        let mut archive_file = String::from("");
        let log_file;

        // INTERNAL FLAGS:
        let mut resume = false;
        let mut have_hash = false;
        let mut user_set_dir= false;


        // Start processing user input


        // Required argument(s):
        // I feel OK using unwrap here because this is a required argument, so clap will have to
        // have received some kind of string here for DuFF to even get to this point.
        let paths = in_args.value_of("dir").unwrap();

        // Split the string from clap into separate directory paths using comma delimiter.
        // Skip checking these files here as we will do that in the next step.
        let path_vec: Vec<String> = paths.split(',').map(|s| s.to_string()).collect();

        // Check each input and parsed directory to make sure its accessible and is a directory.
        for x in path_vec.iter() {

            // Getting a metadata object for the path string the user provided
            // TODO: Write check to see if a dir w/o read perms returns OK or ERR
            // TODO: Should we exit if there is an issue with one of the dirs or just try to contin?
            match fs::metadata(x) {
                Ok(m) => {

                    // Check to see if this is actually a directory and if it is not then send an
                    // error to stderr and exit, if it is a directory we should be good to go.
                    if m.is_dir() == false {
                        let err_str = format!("Specified directory {} is not a directory!",
                                              x);
                        eprintln!("{}", textwrap::fill(err_str.as_str(),
                                                       textwrap::termwidth()));
                        process::exit(1);
                    }
                },
                // If there was some unknown (to me) error then capture it and send it to stderr and
                // exit
                Err(e) => {
                    let err_str = format!("There was an error with the specified directory, {}: {}!",
                                          x, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                }
            }
        }


        // Optional flags:

        if in_args.is_present("archive") {
            archive = true;
        }

        if in_args.is_present("log") {
            log = true;
        }

        if in_args.is_present("prog") {
            hide_prog = true;
        }

        if in_args.is_present("silent") {
            silent = true;

            // If the user wants silence, we're not going to let them see progress, because that
            // wouldn't be silence, would it?
            hide_prog = true;
        }


        // Optional Arguments:

        // Try to capture user input with byte_unit's handy string to Byte function and if byte_unit
        // can understand the user input convert it to bytes, otherwise send an error message to the
        // user
        // TODO: What does a byte_unit error look like for a unit it doesn't comprehend?
        if let Some(ll) = in_args.value_of("lower_lim") {
            match Byte::from_str(ll) {
                Ok(n) => ll_size = n.get_bytes(),
                Err(e) => {
                    let err_str = format!("Lower size limit {}: {}!",
                                          ll, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(),
                                                   textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }

        // Same logic as for handling the lower limit input.
        // TODO: What if the user requests a number over 340 YB?
        if let Some(ul) = in_args.value_of("upper_lim") {
            match Byte::from_str(ul) {
                Ok(n) => ul_size = n.get_bytes(),
                Err(e) => {
                    let err_str = format!("Lower size limit {}: {}!",
                                          ul, e.to_string());
                    eprintln!("{}", textwrap::fill(err_str.as_str(), textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }

        // Try to parse the number of jobs the user specified into a u64 and if it cannot be
        // successfully parsed then just send an error message and die.
        if let Some(n_jobs) = in_args.value_of("jobs") {
            match n_jobs.parse::<u64>() {
                Ok(n) => jobs = n,
                Err(_e) => {
                    let err_str = format!("Number of jobs specificed, {}, is not a valid \
                                                  number!", n_jobs);
                    eprintln!("{}", textwrap::fill(err_str.as_str(),
                                                   textwrap::termwidth()));
                    process::exit(1);
                },
            }
        }


        // Read in the string the user gave us that should contain comma separated extensions they
        // want to require for files and try to split on commas. There aren't really QC checks
        // being done here, but what can be done?
        if let Some(mut in_exts) = in_args.value_of("exts") {

            // Leaving the unwrap as we wouldn't get into this block unless clap gave us something
            // for exts above in the if let Some condition.
            in_exts = in_args.value_of("exts").unwrap();
            exts = in_exts.split(',').map(|s| s.to_string()).collect();
        }

        // See if the user specified an output directory and if so capture it.
        if let Some(out_d) = in_args.value_of("out_dir") {
            out_dir = out_d.to_string();
            user_set_dir = true;

        } else {

            // They didn't give us a output directory so grab the cwd, if that doesn't work die!
            // env::current_dir returns an error if it doesn't exist or user has insufficient perms.
            let cwd = match env::current_dir() {
                Ok(t) => t,
                Err(e) => {
                    let err_str = format!("Could not use the current working directory to \
                                store output, please specify where {} can write the final report \
                                using the -o (--out_dir) argument. \nError text: {}",
                                util::PROG_NAME.to_owned() + util::PROG_VERS, e);
                    eprintln!("{}", textwrap::fill(err_str.as_str(),
                                                   textwrap::termwidth()));

                    std::process::exit(1);
                }
            };

            // Stuff the cwd into a string for out_dir variable
            // TODO: Should we trust unwrap here? We know it will be a PathBuf since its from env.
            out_dir = String::from(cwd.to_str().unwrap());
        }

        // Determine if the output directory path has a trailing / and if so remove it.

        match out_dir.chars().last() {
            Some(t) => {
                if t.to_string() == "/" {
                    out_dir.pop();
                }
            }
            _ => {}
        }



        // If they want us to resume then they need to give us a log file from a previous DuFF run,
        // which we will just take as a string, but should probably do some checks on.
        // TODO: Add some checks to validate this file - maybe not in this function though!
        if let Some(res_f) = in_args.value_of("resume") {
            res_file = res_f.to_string();

            // If they give us a resume file we switch on resume.
            resume = true;
        }

        // If they have precomputed hash file we will load this in and skip calculating hashes for
        // any files listed in this hash file that still have a current m-time.
        // TODO: We should probably also add checks to validate this file - again not here though!
        if let Some(hash_f) = in_args.value_of("hash") {
            prev_hash_file = hash_f.to_string();

            // If they give us a hash file, switch on have_hash
            have_hash = true;
        }


        // Other work

        // Specify the paths for our working files, we'll create them later.
        if archive {

            archive_file = format!("{}/DuFF_{}.arch", out_dir, util::f_dt());
        } else {
            // This is  shitty solution but I need to be able to open a log file even if they don't
            // want it.  If they don't want it I'll leave it hidden and will clean it up.
            archive_file = format ! ("{}/.DuFF_{}.arch", out_dir, util::f_dt());
        }

        if log {

            log_file = format!("{}/DuFF_{}.log", out_dir, util::f_dt());
        } else {
            // This is  shitty solution but I need to be able to open a log file even if they don't
            // want it.  If they don't want it I'll leave it hidden and will clean it up.
            log_file = format ! ("{}/.DuFF_{}.log", out_dir, util::f_dt());
        }

    let report_file = format!("{}/DuFF_{}.report", out_dir, util::f_dt());

        // I know a lot of these can be simplified due to the matching names, but I prefer
        // explicitly specifying the values instead.
        Config {

            // Required argument(s):
            search_path: path_vec,

            // Optional flags:
            archive: archive,
            log: log,
            hide_prog: hide_prog,
            silent: silent,

            // Optional Arguments:
            ll_size: ll_size,
            ul_size: ul_size,
            jobs: jobs,
            exts: exts,
            out_dir: out_dir,
            res_file: res_file,
            prev_hash_file: prev_hash_file,
            
            // INTERNAL ARGUMENTS:
            archive_file: archive_file,
            log_file: log_file,
            report_file: report_file,

            // INTERNAL FLAGS:
            resume: resume,
            have_hash: have_hash,
            user_set_dir: user_set_dir,

        }
    }
}

// Implementing the Display trait so that we can easily print out the DuFF configuration both out
// to stdout as well as to a log file if needed.
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // Using this border string to make a box around the DuFF header.
        let border_str = "=".repeat(textwrap::termwidth());
        let mut out_str = format!("{}\n", border_str).to_owned();
        out_str.push_str(format!("{:<21} {:^39} {:>0}\n", util::dt(), "Overview",
                                 util::PROG_NAME.to_owned() + " v" + util::PROG_VERS).as_str());
        out_str.push_str(format!("{}\n", border_str).as_str());

        if self.resume {
            out_str.push_str(format!("{:<40} {:>1}\n", "Status:", "Resuming" ).as_str());
        } else {
            out_str.push_str(format!("{:<40} {:>1}\n", "Status:", "New Run" ).as_str());
        }

        out_str.push_str(format!("{:<40} {:>1}\n", "Search Directories:",
                                 self.search_path.join(",")).as_str());

        out_str.push_str(format!("{:<40} {:>1}\n", "Extensions:", self.exts.join(", "))
                .as_str());


        // Use the default value of ll to determine if the user specified one and if they did we
        // print it out, if they didn't we don't print it out so as to not confuse the user.
        if self.ll_size > 0 {
            let ll_str = converter::convert(self.ll_size as f64);
            out_str.push_str(format!("{:<40} {:>1}\n", "Minimum Size:", ll_str).as_str());

        }

        // Handled in the same manner as ll_size, except now comparing to max u128 value.
        if self.ul_size < 340282366920938463463374607431768211455 {
            let ul_str = converter::convert(self.ul_size as f64);
            out_str.push_str(format!("{:<40} {:>1}\n", "Maximum Size:", ul_str).as_str());

        }

        out_str.push_str(format!("{:<40} {:>1}\n", "Number of Threads:", self.jobs).as_str());
        out_str.push_str(format!("{:<40} {:>1}\n", "Output Directory:", self.out_dir).as_str());
        out_str.push_str(format!("{:<40} {:>1}\n", "Final Report:", self.report_file).as_str());

        if self.archive {
            out_str.push_str(format!("{:<40} {:>1}\n", "Save Hashes:", self.archive_file).as_str());

        } else {
            out_str.push_str(format!("{:<40} {:>1}\n", "Save Hashes:", self.archive).as_str());
        }

        if self.log {
            out_str.push_str(format!("{:<40} {:>1}\n", "Save Log:", self.log_file).as_str());

        } else {
            out_str.push_str(format!("{:<40} {:>1}\n", "Save Log:", self.log).as_str());
        }

        if self.hide_prog == true {
            out_str.push_str(format!("{:<40} {:>1}\n", "Hide Progress:", self.hide_prog)
                .as_str());
        }

        out_str.push_str(format!("{:<40} {:>1}\n", "Report any issues at:", util::PROG_ISSUES)
                .as_str());

        write!(f, "{}", out_str)
    }
}