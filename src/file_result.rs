use std::cmp::Ordering;
use std::fs::File;
use std::time;
use std::io::{BufReader, BufRead};

use std::hash::Hasher;
use twox_hash::xxh3::{Hash128, HasherExt};


// struct to hold file information
#[derive(Debug, Clone,Eq)]
pub struct FileResult {
    pub file_path : String,
    pub size : u128,
    pub mtime: time::SystemTime,
    pub hash : String
}

impl FileResult {
    pub fn new(file_path: String, size: u128, mtime: time::SystemTime) -> FileResult {
        FileResult {file_path, size, mtime, hash : String::new()}
    }


    pub fn calc_hash(&mut self, buff_size: usize) {

        // Open the file
        let mut f = File::open(&self.file_path).unwrap();

        // Create the buff reader
        let mut f = BufReader::with_capacity(buff_size, f);

        // Create the hasher - at some point maybe give them options?
        // Thanks to  Jake Goulding for the help! https://stackoverflow.com/a/48534068
        let mut hasher = Hash128::default();
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
        let hash = hasher.finish_ext().to_string();

        self.update_hash(hash);

    }

    pub fn update_hash(&mut self, hash: String)  {
        self.hash = hash;
    }
}


// Adapted from SO answer: https://stackoverflow.com/a/29884582
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