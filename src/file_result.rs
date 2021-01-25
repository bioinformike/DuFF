use std::cmp::Ordering;

// struct to hold file information
#[derive(Debug, Clone,Eq)]
pub struct FileResult {
    pub file_path : String,
    pub size : u64,
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