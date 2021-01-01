// struct to hold file information
#[derive(Debug, Clone)]
pub struct FileResult {
    file_path : String,
    size : u64,
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
