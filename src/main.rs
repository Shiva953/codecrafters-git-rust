#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::prelude::*;

//[CONTINUATION PROJECT] - IMPLEMENTATING GIT FROM SCRATCH
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    } else if (args[1] == "cat-file") {
        // read the blob object

        // HOW TO IDENTIFY A BLOB FILE IN THE FIRST PLACE?
        // THIS IS HOW ITS CONTENTS LOOK AFTER DECOMPRESSION
        // blob <size>\0<content>

        // Step 1: identify the file & read from it
        //object directory is in form of .git/objects/[first 2 hash digits]/[remaining hash digits after that]
        //ex - .git/objects/e8/8f7a929cd70b0274c4ea33b209c97fa845fdbc
        let content = fs::read(format!(".git/objects/{}/{}", &args[3][..2], &args[3][2..])).unwrap();

        //Step2: Decompress using Zlib
        let mut decompressed_data = ZlibDecoder::new(&content[..]);

        //Step3: EXTRACT CONTENT from the DECOMPRESSED DATA
        let mut blob_file_contents_vec = Vec::new();
        decompressed_data.read_to_end(&mut blob_file_contents_vec) ;//filling the buffer with contents of blob file
        //buffer needs to be converted to string
        let readable_blob = String::from_utf8(blob_file_contents_vec).unwrap();
        // now extract <content> from blob <size>\0<content>
        match extract_content(&readable_blob) {
            Some(content) => print!("{}", content.to_string()),
            None => print!("No content found"),
        }

        //create the blob object
    }
    else {
        println!("unknown command: {}", args[1])
    }
}

fn extract_content(input: &str) -> Option<&str> {
    // Find the position of the null byte
    if let Some(null_pos) = input.find('\0') {
        // Return the substring after the null byte
        Some(&input[null_pos + 1..])
    } else {
        None
    }
}

