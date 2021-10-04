use log;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_bin_file(path: &Path) -> Result<Vec<u8>, ()> {
    let file = File::open(path);
    if file.is_err() {
        log::error!("Failed to open file {}", path.display());
        return Err(());
    }

    let mut buffer = vec![];
    match file.unwrap().read_to_end(&mut buffer) {
        Ok(_) => return Ok(buffer),
        Err(_) => {
            log::error!("Failed to read file data {}", path.display());
            return Err(());
        }
    }
}
