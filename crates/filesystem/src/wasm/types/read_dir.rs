
use std::io;

pub struct ReadDir {

}

impl ReadDir {}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        todo!()
    }
}

pub struct DirEntry {

}

impl DirEntry {
    pub fn path(&self) -> std::path::PathBuf {
        todo!()
    }
}