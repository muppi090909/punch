use std::io::Read;
use std::io::Write;
use std::{fs, io::BufWriter, path::Path};

// Reusable functions should be added to this file
pub fn move_file(from: &Path, to: &Path) {
    let mut f = fs::File::open(&from).unwrap();
    let mut file_buffer = Vec::new();
    f.read_to_end(&mut file_buffer).unwrap();

    let mut dest_file_buffer = BufWriter::new(fs::File::create(to).unwrap());
    dest_file_buffer.write_all(&file_buffer).unwrap();
    dest_file_buffer.flush().unwrap();
}

pub fn create_file(file: &Path) {
    fs::File::create(file).expect(format!("error creating file: {:?}", file).as_str());
}

pub fn create_directory(dir: &Path) {
    fs::create_dir_all(dir).expect(format!("error creating folder: {:?}", dir).as_str());
}

pub fn remove_file(file: &Path) {
    fs::remove_file(file).expect(format!("error deleting folder: {:?}", file).as_str());
}

pub fn remove_directory(dir: &Path) {
    fs::remove_dir_all(dir).expect(format!("error deleting folder: {:?}", dir).as_str());
}
