#[allow(unused_imports)]
use std::env;
use std::fmt::format;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::result;
use anyhow::Error;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use sha1::{Digest, Sha1};
use std::os::unix::fs::PermissionsExt;

//[CONTINUATION PROJECT] - IMPLEMENTATING GIT FROM SCRATCH

//TODO - UNDERSTAND PARSING + IMPLEMENT WRITE-TREE
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
        //improve error handling
        decompressed_data.read_to_end(&mut blob_file_contents_vec).unwrap() ;//filling the buffer with contents of blob file
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
        let path = &args[3];
        // let path = format!("./{}", filename);
        let content = fs::read_to_string(path.clone());
        let sha_input = match fs::read_to_string(path.clone()){ //BLOB FILE CONTENT
            Ok(x) => {
                let size = fs::metadata(path.clone()).unwrap().len();
                format!("blob {}\0{}", size, x)
            },
            Err(e) => {
                "invalidfilecontent".to_string()
            }
        };
        // print!("sha_input {}", sha_input);

        // STEP 2 - COMPUTE THE SHA1HASH(blob <size>\0<original file contents>)
        let mut hasher = Sha1::new();
        hasher.update(sha_input.clone());
        let hash_result = hasher.finalize();

        let hash = format!("{:x}", hash_result);
        print!("{}", &hash);


        // STEP 3 - CREATE THE .git/objects/[hash[..2]]/[hash[2..]] file
        let dir_name = &hash[..2];
        let file_name = &hash[2..];

        // Create the full path .git/objects/[hash[..2]]/[hash[2..]]
        let dir_path = format!(".git/objects/{}", dir_name);
        let file_path = format!("{}/{}", dir_path, file_name);
        // create directory for blob object
        fs::create_dir_all(&dir_path).unwrap();

        // Create blob object file
        let mut blob_object_file = fs::File::create(file_path).unwrap();

        // STEP 4 - COMPRESS THE CONTENTS OF THE ORIGINAL FILE USING ZLIB AND WRITE IT TO THE .git/objects/[hash[..2]]/[hash[2..]] file
        let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
        encoder.write_all(sha_input.as_bytes()).unwrap();
        encoder.finish().unwrap();
    }
    else if (args[1] == "ls-tree") {
        // LS-TREE COMMAND IMPLEMENTATION, GET IT!
        //git ls-tree --name-only <tree_sha>

        // STEP1: GET THE TREE CONTENTS FROM THE TREE_SHA(TREE_SHA -> TREE OBJECT -> COMPRESSED CONTENT -> TREE CONTENTS(IN STRING FORMAT))
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

        //IMPROVE ERROR HANDLING
        let entries = parse_tree_object(&tree_file_contents_vec);
        // print!("tree string: {}", readable_tree);
        // let entries = extract_content_from_tree_string(&readable_tree);

        for name in entries {
            println!("{}", name);
        }

    }
    else if (args[1] == "write-tree") {
        // WRITE-TREE COMMAND IMPLEMENTATION
        // The git write-tree command creates a tree object from the current state of the "staging area". 
        // so it takes the staging area files in the WORKING DIRECTORY into consideration
        // STEP 1: FIND THE WORKING DIR + ITERATE OVER ALL FILES/DIRS IN THE WORKING DIR
        // STEP 2: IF(FILE) -> CREATE BLOB OBJECT, IF(DIR) -> CREATE TREE OBJECT RECURSIVELY, RECORD SHA HASH
        // BLOB HASH = SHA(UNCOMPRESSED FILE CONTENT)
        // TREE HASH = SHA(DIRECTORY CONTENT) = SHA(FILE CONTENTS + DIR CONTENT) = SHA(FILE CONTENTS + SHA(FILES + DIRS INSIDE IT)) = ...
        // STEP 3: CREATE OBJECT in .git/objects

        // let staging_area = Path::read_dir("./").unwrap();
        let working_dir = env::current_dir().expect("Failed to get current directory");
        let working_dir_entries = std::fs::read_dir(".").expect("Failed to read directory");
        let final_hash = Sha1::new();
        
         let hashes:Vec<String> = Vec::new();
        //  let mut content = Vec::new();
         let mut files: Vec<(String, String)> = Vec::new(); //[...(file_name,mode)]

        if let Ok(entries) = fs::read_dir(".") { //entries in the current working directory
        for entry in entries{
            if let Ok(entry) = entry {
                //mode, hash and name needs to be appended to the string which will be compresses
                let file_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                if !file_name.starts_with(".git") {
                    let permissions = entry.metadata().unwrap().permissions();
                    let mode = format!("{:0>6o}", permissions.mode());
                    files.push((file_name, mode));
                    }
                }
            }
        }
        files.sort_by(|x, y| x.0.cmp(&y.0));
        let total_size: usize = files.iter()
        .map(|(name, mode)| mode.len() + name.len() + 21) // 20 bytes for SHA-1 + 1 byte for null
        .sum();



        let tree_hash_opt = create_tree(&files);
        if let Ok(tree_hash) = tree_hash_opt{
            // let mut tree_content = Vec::new();
            // let tree_object_path = format!(".git/objects/{}/{}", &tree_hash[..2], &tree_hash[2..]);
            // let header = format!("tree {}\0", tree_content.len()).into_bytes();
            // let mut object_content = [header, tree_content].concat();
            // let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            // encoder.write_all(&object_content).unwrap();
            // let compressed = encoder.finish().unwrap();
            
            // let object_path = format!(".git/objects/{}/{}", &tree_hash[..2], &tree_hash[2..]);
            // // fs::create_dir_all(object_path.unwrap())?;
            // fs::write(object_path, compressed).unwrap();

            print!("{}", tree_hash);
        }

        //the final string must be of given format
        //content of the tree object should be in the following form
        /*
        tree <size>\0
        <mode> <name>\0<20_byte_sha>
        <mode> <name>\0<20_byte_sha>
         */


        //get the final hash and write the file to .git/objects
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

fn parse_tree_object(content: &[u8]) -> Vec<String> {
    let mut entries = Vec::new();
    let mut i = 0;

    // Skip the header (everything before the first null byte)
    while i < content.len() && content[i] != 0 {
        i += 1;
    }
    i += 1; // Skip the null byte

    // Parse entries
    while i < content.len() {
        let mut name = Vec::new();
        // Skip mode
        while i < content.len() && content[i] != b' ' {
            i += 1;
        }
        i += 1; // Skip space
        // Read name
        while i < content.len() && content[i] != 0 {
            name.push(content[i]);
            i += 1;
        }
        i += 1; // Skip null byte
        i += 20; // Skip SHA (20 bytes)

        if let Ok(name_str) = String::from_utf8(name) {
            entries.push(name_str);
        }
    }

    entries
}

fn create_blob(path: &Path) -> Result<String, Error> {
    if !path.is_file() {
        return Err(Error::new(std::io::ErrorKind::InvalidInput));
    }

    let content = fs::read(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let hash = format!("{:x}", hasher.finalize());
    
    let object_content = format!("blob {}\0", content.len()).into_bytes();
    let object_content = [object_content, content].concat();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&object_content)?;
    let compressed = encoder.finish()?;
    
    let object_path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    fs::create_dir_all(Path::new(&object_path).parent().unwrap())?;
    fs::write(object_path, compressed)?;
    
    Ok(hash)
}

fn create_tree(files: &[(String, String)]) -> Result<String, Error> {
    let mut tree_content = Vec::new();
    
    for (name, mode) in files {
        let path = Path::new(name);
        let hash = create_blob(path).map_err(|e| {
            Error::new(e.kind())
        })?;
        
        let entry = format!("{} {}\0", mode, name).into_bytes();
        tree_content.extend_from_slice(&entry);
        tree_content.extend_from_slice(&hex::decode(hash).map_err(|e| {
            Error::new(std::io::ErrorKind::InvalidData)
        })?);
    }
    
    let mut hasher = Sha1::new();
    hasher.update(&tree_content);
    let hash = format!("{:x}", hasher.finalize());
    
    let header = format!("tree {}\0", tree_content.len()).into_bytes();
    let object_content = [header, tree_content].concat();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&object_content)?;
    let compressed = encoder.finish()?;
    
    let object_path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    fs::create_dir_all(Path::new(&object_path).parent().unwrap())?;
    fs::write(object_path, compressed)?;
    
    Ok(hash)
}