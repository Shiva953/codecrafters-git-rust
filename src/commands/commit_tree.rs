use std;
use sha1::{Digest, Sha1};
use std::fs;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

pub struct CommitTree;

impl CommitTree{
    pub fn run(args: &[String]) -> Result<(), String>{

        let message = format!("{}\n", args[4]);
        let parent_commit_sha = &args[2];
        let tree_sha = &args[0];

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
        // let mut encoder = ZlibEncoder::new(commit_object_file, Compression::default());
        // encoder.write_all(commit_obj_str.as_bytes()).unwrap();
        // encoder.finish().unwrap();
        let mut encoder = ZlibEncoder::new(commit_object_file, Compression::default());
        encoder.write_all(commit_obj_str.as_bytes()).map_err(|e| e.to_string())?;
        encoder.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}