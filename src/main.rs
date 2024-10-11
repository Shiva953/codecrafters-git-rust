#[allow(unused_imports)]
use std::env;
use std::fmt::format;
#[allow(unused_imports)]
use std::fs;
use anyhow::Error;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use sha1::{Digest, Sha1};

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
    } else if args[1] == "cat-file" {
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

    } else if args[1] == "hash-object" {
        // replicating command "git hash-object -w hello.txt"
        // create the blob file
        // ex file hello.txt

        // STEP 1 - TAKE ORIGINAL CONTENTS OF EXAMPLE FILE
        let path = &args[3];
        // let path = format!("./{}", filename);
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
        let blob_object_file = fs::File::create(file_path).unwrap();

        // STEP 4 - COMPRESS THE CONTENTS OF THE ORIGINAL FILE USING ZLIB AND WRITE IT TO THE .git/objects/[hash[..2]]/[hash[2..]] file
        let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
        encoder.write_all(sha_input.as_bytes()).unwrap();
        encoder.finish().unwrap();
    }
    else if args[1] == "ls-tree" {
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
    else if args[1] == "write-tree" {
        let tree_hash = write_tree(".");
        print!("{}", tree_hash);
        
    }
    else if args[1] == "commit-tree"{
        // commit object:
        // git commit <tree_sha> -p <parent_commit_sha> -m <message>

        let message = format!("{}\n", args[6]);
        let parent_commit_sha = &args[4];
        let tree_sha = &args[2];

        let now = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).expect("Time went backwards");
        let timestamp = now.as_secs();
        
        // Hardcoded values for demonstration (replace these with actual values in a real implementation)
        let author_name = "John Doe";
        let author_email = "johndoe@example.com";
        let committer_name = "John Doe";
        let committer_email = "johndoe@example.com";
        let timezone = "+0000";

        // Construct the content of the commit object
        let content = format!(
            "tree {}\n\
            parent {}\n\
            author {} <{}> {} {}\n\
            committer {} <{}> {} {}\n\
            \n\
            {}",
            tree_sha,
            parent_commit_sha,
            author_name, author_email, timestamp, timezone,
            committer_name, committer_email, timestamp, timezone,
            message,
        );


        let size = content.len();

        let commit_obj_str = format!("commit {}\0{}", size, content);
        let mut hasher = Sha1::new();
        hasher.update(commit_obj_str.clone());
        let hash_result = hasher.finalize();

        let commit_hash = format!("{:x}", hash_result);
        print!("{}", &commit_hash);

        let dir_name = &commit_hash[..2];
        let file_name = &commit_hash[2..];

        let dir_path = format!(".git/objects/{}", dir_name);
        let file_path = format!("{}/{}", dir_path, file_name);

        fs::create_dir_all(&dir_path).unwrap();


        let commit_object_file = fs::File::create(file_path).unwrap();

        // STEP 4 - COMPRESS THE CONTENTS OF THE ORIGINAL FILE USING ZLIB AND WRITE IT TO THE .git/objects/[hash[..2]]/[hash[2..]] file
        let mut encoder = ZlibEncoder::new(commit_object_file, Compression::default());
        encoder.write_all(commit_obj_str.as_bytes()).unwrap();
        encoder.finish().unwrap();
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

// pub fn hash_object<P: AsRef<std::path::Path>>(
//     path: P,
//     object_type: &str,
//     contents: &[u8],
// ) -> String {
//     use std::fs::File;
//     use std::io::Write;
//     let mut object = Vec::new();
//     object.extend_from_slice(object_type.as_bytes());
//     object.extend_from_slice(&[b' ']);
//     object.extend_from_slice(contents.len().to_string().as_bytes());
//     object.extend_from_slice(&[b'\0']);
//     object.extend_from_slice(&contents);
//     let compressed_contents = compress(&object);
//     let sha = sha_hash_str(&object);
//     let sha_dir = object_dir(&path, &sha);
//     std::fs::create_dir(&sha_dir).expect("Failed to create sha object directory");
//     let mut file =
//         File::create(object_file(path, &sha)).expect("Failed to create test object file");
//     file.write_all(&compressed_contents)
//         .expect("Failed to write test object file");
//     sha
// }

// fn write_tree(path: &str) -> String {
//     let mut entries = Vec::new();
//     for entry in std::fs::read_dir(path).expect("Failed to read directory") {
//         let entry = entry.expect("Failed to read directory entry");
//         let file_name = entry.file_name().into_string().unwrap();
//         if file_name == ".git" { continue; }
        
//         let file_type = entry.file_type().expect("Failed to get file type");
//         let mode = if file_type.is_dir() { "40000" } else { "100644" };
        
//         let (hash, raw_hash) = if file_type.is_dir() {
//             let subtree_hash = write_tree(&entry.path().to_str().unwrap());
//             (subtree_hash.clone(), hex::decode(subtree_hash).unwrap())
//         } else {
//             let contents = std::fs::read(entry.path()).expect("Failed to read file");
//             hash_object("blob", &contents)
//         };
        
//         entries.push((mode, file_name, raw_hash));
//     }
    
//     entries.sort_by(|a, b| a.1.cmp(&b.1));
    
//     let mut tree_content = Vec::new();
//     for (mode, name, hash) in entries {
//         tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
//         tree_content.extend_from_slice(&hash);
//     }
    
//     let (tree_hash, _) = hash_object("tree", &tree_content);
//     write_object(path, "tree", &tree_content)
// }

// pub fn hash_object<P: AsRef<std::path::Path>>(
//     path: P,
//     object_type: &str,
//     contents: &[u8],
// ) -> String {
//     use std::fs::File;
//     use std::io::Write;
//     use flate2::write::ZlibEncoder;
//     use flate2::Compression;
//     use sha1::{Sha1, Digest};

//     // STEP 1 - Prepare the content with header
//     let size = contents.len();
//     let mut sha_input = Vec::new();
//     sha_input.extend_from_slice(object_type.as_bytes());
//     sha_input.extend_from_slice(b" ");
//     sha_input.extend_from_slice(size.to_string().as_bytes());
//     sha_input.extend_from_slice(b"\0");
//     sha_input.extend_from_slice(contents);

//     // STEP 2 - Compute the SHA1 hash
//     let mut hasher = Sha1::new();
//     hasher.update(&sha_input);
//     let hash_result = hasher.finalize();
//     let hash = format!("{:x}", hash_result);

//     // STEP 3 - Create the .git/objects/[hash[..2]]/[hash[2..]] file
//     let dir_name = &hash[..2];
//     let file_name = &hash[2..];
//     let dir_path = path.as_ref().join(".git").join("objects").join(dir_name);
//     let file_path = dir_path.join(file_name);

//     std::fs::create_dir_all(&dir_path).expect("Failed to create object directory");

//     // STEP 4 - Compress the contents and write to the object file
//     let blob_object_file = File::create(file_path).expect("Failed to create object file");
//     let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
//     encoder.write_all(&sha_input).expect("Failed to write compressed object");
//     encoder.finish().expect("Failed to finish compression");

//     hash
// }

pub fn hash_object(object_type: &str, contents: &[u8]) -> (String, Vec<u8>) {
    use sha1::{Sha1, Digest};

    let size = contents.len();
    let mut sha_input = Vec::new();
    sha_input.extend_from_slice(object_type.as_bytes());
    sha_input.extend_from_slice(b" ");
    sha_input.extend_from_slice(size.to_string().as_bytes());
    sha_input.extend_from_slice(b"\0");
    sha_input.extend_from_slice(contents);

    let mut hasher = Sha1::new();
    hasher.update(&sha_input);
    let hash_result = hasher.finalize();
    
    (format!("{:x}", hash_result), hash_result.to_vec())
}

fn write_tree(path: &str) -> String {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let file_name = entry.file_name().into_string().unwrap();
        if file_name == ".git" { continue; }
        
        let file_type = entry.file_type().expect("Failed to get file type");
        let mode = if file_type.is_dir() { "40000" } else { "100644" };
        
        let raw_hash = if file_type.is_dir() {
            let subtree_hash = write_tree(&entry.path().to_str().unwrap());
            hex::decode(subtree_hash).unwrap()
        } else {
            let contents = std::fs::read(entry.path()).expect("Failed to read file");
            let (_, raw_hash) = hash_object("blob", &contents);
            raw_hash
        };
        
        entries.push((mode, file_name, raw_hash));
    }
    
    entries.sort_by(|a, b| a.1.cmp(&b.1));
    
    let mut tree_content = Vec::new();
    for (mode, name, hash) in entries {
        tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
        tree_content.extend_from_slice(&hash);
    }
    
    write_object(path, "tree", &tree_content)
}

pub fn write_object<P: AsRef<std::path::Path>>(path: P, object_type: &str, contents: &[u8]) -> String {
    use std::fs::File;
    use std::io::Write;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let (hash, _) = hash_object(object_type, contents);

    // Create the .git/objects/[hash[..2]]/[hash[2..]] file
    let dir_name = &hash[..2];
    let file_name = &hash[2..];
    let dir_path = path.as_ref().join(".git").join("objects").join(dir_name);
    let file_path = dir_path.join(file_name);

    std::fs::create_dir_all(&dir_path).expect("Failed to create object directory");

    // Prepare the content with header
    let size = contents.len();
    let mut object_content = Vec::new();
    object_content.extend_from_slice(object_type.as_bytes());
    object_content.extend_from_slice(b" ");
    object_content.extend_from_slice(size.to_string().as_bytes());
    object_content.extend_from_slice(b"\0");
    object_content.extend_from_slice(contents);

    // Compress the contents and write to the object file
    let blob_object_file = File::create(file_path).expect("Failed to create object file");
    let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
    encoder.write_all(&object_content).expect("Failed to write compressed object");
    encoder.finish().expect("Failed to finish compression");

    hash
}

fn sha_hash(data: &[u8]) -> [u8; 20] {
    use std::convert::TryInto;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    hasher.finalize().as_slice().try_into().unwrap()
}

fn sha_hash_str(data: &[u8]) -> String {
    hex::encode(sha_hash(data))
}

fn object_dir<P: AsRef<std::path::Path>>(repo_dir: P, sha: &str) -> std::path::PathBuf {
    repo_dir
        .as_ref()
        .join(".git")
        .join("objects")
        .join(&sha[..2])
}

fn object_file<P: AsRef<std::path::Path>>(repo_dir: P, sha: &str) -> std::path::PathBuf {
    object_dir(repo_dir, sha).join(&sha[2..])
}

fn compress(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .expect("Failed to write object type to encoder");
    encoder.finish().expect("Failed to finish compression")
}
fn decompress(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibDecoder;
    use std::io::Write;
    let mut decoder = ZlibDecoder::new(Vec::new());
    decoder
        .write_all(data)
        .expect("Failed to write object type to decoder");
    decoder.finish().expect("Failed to finish decompression")
}