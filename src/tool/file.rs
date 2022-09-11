use std::fs::File;
use std::io::Read;

use anyhow::Result;

///读取文件，并返回str
pub fn read_file_to_str(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut str = String::new();
    file.read_to_string(&mut str)?;
    Ok(str)
}

