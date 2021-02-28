// Implement Ord, PartialOrd, and PartialEq traits so HashMap can determine if 2 FileResults structs
// are equal (considering file size and the file hash, although as DuFF is currently written, I
// believe that these traits are only used before a hash is even calculated, defaults to "", so
// really it's only considering file size, but the functionality is implemented to consider hash
// values as well).
use std::cmp::Ordering;

// For opening files to hash
use std::fs::File;

// For implementation of Display trait
use std::fmt;

// For reading a file in and hashing it
use std::io::{BufReader, BufRead};

// For storage of mtime
use chrono::{DateTime, Utc};

// Used for the hasher write function
use std::hash::Hasher;

// For XX3_128 bit hashing.
use twox_hash::xxh3::{Hash128, HasherExt};



// The FileResult struct will hold information for each file of potential interest to us.  A struct
// will only be created if the file represented by the file_path meets both any user specified lower
// or upper file size limit and any user specified extension requirements.
#[derive(Debug, Clone,Eq)]
pub struct FileResult {

    // The full path to the file, held as a String for convenience.
    pub file_path : String,

    // The size of this file as a u128 to future proof us (hopefully).
    pub size : u128,

    // The mtime or time this file was last modified.  This will be used to determine if we need to
    // recacluate a hash when the user provides us with an old set of hashes using the hash argument
    pub mtime: DateTime<Utc>,

    // The XXX3 128-bit hash stored as a string for convenience.
    pub hash : String
}

impl FileResult {

    // Simple new function, note that with how DuFF currently functions we do not have a hash when
    // the FileResult object is first created, so we set it to an empty string here. There is an
    // update_hash function below that allows us to update the hash later after we calculate it.
    pub fn new(file_path: String, size: u128, mtime: DateTime<Utc>) -> FileResult {
        FileResult {file_path, size, mtime, hash : String::new()}
    }

    // The calc_hash function does what it says its going to do, calculate a hash, specifically as
    // currently implemented a(n) XXH3 128-bit hash. It does not return a value, instead directly
    // updating the hash associated with self.
    // Arguments are as follows:
    // buff_size: The size of the BuffReader buffer capacity, as a usize.  This was used when doing
    //            testing, but for now at least there is a hard-coded value in the main code, so
    //            currently this argument is of little value.
    pub fn calc_hash(&mut self, buff_size: usize) {

        // Open the file for reading to hash it
        // TODO: Handle this unwrap!
        let f = File::open(&self.file_path).unwrap();

        // Create the BufReader for the file f with supplied buff_size.
        let mut f = BufReader::with_capacity(buff_size, f);

        // "Adapted" from https://stackoverflow.com/a/48534068
        // Thanks to Jake Goulding for this answer and the additional help!

        // Create the actual hasher
        let mut hasher = Hash128::default();

        // Loop until we run out of file, hashing as we go!
        loop {
            let consumed = {
                let bytes = f.fill_buf().unwrap();
                if bytes.is_empty() {
                    break;
                }
                hasher.write(bytes);
                bytes.len()
            };
            f.consume(consumed);
        }

        // Finish off the hash and store it as a string.
        let hash = hasher.finish_ext().to_string();

        // Update self's hash variable with this newly calculated hash function.
        self.update_hash(hash);

    }

    // The update_hash function simply takes a string and sets that string as the hash for the file
    // represented by this (self) FileResult object. It does not return a value and does no QC of
    // the supplied hash.
    // Arguments are as follows:
    // hash: The string to update this (self) objects hash value to.
    pub fn update_hash(&mut self, hash: String)  {
        self.hash = hash;
    }
}

// Implement the Display Trait for easy printing of FileResult objects
impl fmt::Display for FileResult {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        write!(f, "{} {} {} {}", self.size, self.hash, self.mtime, self.file_path)
    }
}

// Implementing Ord, PartialOrd, and PartialEq for the FileResult struct to be used when inserting
// them into hashmaps. This is "adapted" from SO answer: https://stackoverflow.com/a/29884582 and
// uses both the FileResult's file size value as well as the hash string for comparison.
impl Ord for FileResult {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.size, &self.hash).cmp(&(other.size, &other.hash))
    }
}

impl PartialOrd for FileResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileResult {
    fn eq(&self, other: &Self) -> bool {
        (self.size, &self.hash) == (other.size, &other.hash)
    }
}