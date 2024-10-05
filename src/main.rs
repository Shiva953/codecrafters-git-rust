#[allow(unused_imports)]
use std::env;
use std::fmt::format;
#[allow(unused_imports)]
use std::fs;
use std::result;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use sha1::{Digest, Sha1};

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
        //command = git cat-file -p <blobhash>, args[3] = blobhash, args[3][..2] = first 2 digits(blob object folder), args[3][2..] = actual blob object FILE
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

    } else if(args[1] == "hash-object"){
        // replicating command "git hash-object -w hello.txt"
        // create the blob file
        // ex file hello.txt

        // STEP 1 - TAKE ORIGINAL CONTENTS OF EXAMPLE FILE
        let filename = args[3];
        let path = format!("./{}", filename);
        let content = fs::read_to_string(path.clone());
        let sha_input = match fs::read_to_string(path.clone()){
            Ok(x) => {
                let size = fs::metadata(path.clone())?.len();
                format!("blob {}\0{}", size, x)
            },
            Err(e) => {
                "invalidfilecontent".to_string()
            }
        };

        // STEP 2 - COMPUTE THE SHA1HASH(blob <size>\0<original file contents>)
        let mut hasher = Sha1::new();
        hasher.update(sha_input.clone());
        let mut hash = hasher.finalize();

        // STEP 3 - CREATE THE .git/objects/[hash[..2]]/[hash[2..]] file
        let dir_name = &hash[..2];
        let file_name = &hash[2..];
        // Create the full path .git/objects/[hash[..2]]/[hash[2..]]
        let dir_path = format!(".git/objects/{}", dir_name);
        let file_path = format!("{}/{}", dir_path, file_name);
        // create directory for blob object
        fs::create_dir_all(&dir_path)?;
        // Create blob object file
        let mut blob_object_file = fs::File::create(file_path)?;

        // STEP 4 - COMPRESS THE CONTENTS OF THE ORIGINAL FILE USING ZLIB AND WRITE IT TO THE .git/objects/[hash[..2]]/[hash[2..]] file
        let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
        encoder.write_all(content.unwrap().as_bytes());
        encoder.finish()?;
    }
    else if (args[1] == "ls-tree") {
        // LS-TREE COMMAND IMPLEMENTATION, GET IT!
        //git ls-tree --name-only <tree_sha>

        //STEP1 : GET THE TREE CONTENTS FROM THE TREE_SHA(TREE_SHA -> TREE OBJECT -> COMPRESSED CONTENT -> TREE CONTENTS(IN STRING FORMAT))
        // STEP2: EXTRACT (FILES, DIRS) FROM THE STRING OF GIVEN FORM + STDOUT

        let tree_sha = &args[3]; //TREE OBJECT IF DIR, BLOB OBJECT IF FILE

        //DECOMPRESS THE FILE CONTENTS OF THE BELOW PATH
        let path = format!(".git/objects/{}/{}", &tree_sha[..2], &tree_sha[2..]);
        let content = fs::read(path).unwrap();
        let mut decompressed_data = ZlibDecoder::new(&content[..]);
        let mut tree_file_contents_vec = Vec::new();
        decompressed_data.read_to_end(&mut tree_file_contents_vec);
        //final string of the form
        /*
        tree <size>\0
        <mode> <name>\0<20_byte_sha>
        <mode> <name>\0<20_byte_sha>
         */
        let readable_tree = String::from_utf8(tree_file_contents_vec).unwrap();
        let final_output = extract_content_from_tree_string(input);

        match final_output.clone() {
            Some(x) => {
                print!(x)
            },
            None => print!("Invalid tree object")
        }

        //TEST THIS(GET MEMBERSHIP) + ITERATE
    }
    else if (args[1] == "write-tree") {
        // WRITE-TREE COMMAND IMPLEMENTATION
        
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

fn extract_content_from_tree_string(input: &str) -> Option<String> {
    let mut result = String::new();
    
    // Skip the first line
    let content = input.splitn(2, '\0').nth(1)?;
    
    for line in content.split('\0') {
        let name = line.split_whitespace().nth(1)?;
        result.push_str(name);
        result.push('\n');
    }
    
    // Remove the last newline if it exists
    if result.ends_with('\n') {
        result.pop();
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

