use std::fs;
use flate2::read::ZlibDecoder;
use std::io::prelude::*;

pub struct CatFile;

impl CatFile {
    pub fn run(args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("Usage: cat-file <blob_hash>".to_string());
        }
        let blob_hash = &args[1];
        let content = fs::read(format!(".git/objects/{}/{}", &blob_hash[..2], &blob_hash[2..]))
            .map_err(|e| e.to_string())?;
        
        let mut decompressed_data = ZlibDecoder::new(&content[..]);
        let mut blob_file_contents_vec = Vec::new();
        decompressed_data.read_to_end(&mut blob_file_contents_vec)
            .map_err(|e| e.to_string())?;
        
        let readable_blob = String::from_utf8(blob_file_contents_vec)
            .map_err(|e| e.to_string())?;
        match Self::extract_content(&readable_blob) {
            Some(content) => print!("{}", content),
            None => print!("No content found"),
        }
        Ok(())
    }

    fn extract_content(input: &str) -> Option<&str> {
        input.find('\0').map(|null_pos| &input[null_pos + 1..])
    }
}