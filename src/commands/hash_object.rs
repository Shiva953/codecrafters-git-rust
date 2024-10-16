use std::fs;
use sha1::{Digest, Sha1};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

pub struct HashObject;

impl HashObject {
    pub fn run(args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("Usage: hash-object <filename>".to_string());
        }
        let filename = &args[0];
        let sha_input = match fs::read_to_string(filename) {
            Ok(x) => {
                let size = fs::metadata(filename).map_err(|e| e.to_string())?.len();
                format!("blob {}\0{}", size, x)
            },
            Err(_) => return Err("Invalid file content".to_string()),
        };

        let mut hasher = Sha1::new();
        hasher.update(sha_input.clone());
        let hash_result = hasher.finalize();

        let hash = format!("{:x}", hash_result);
        print!("{}", &hash);

        let dir_name = &hash[..2];
        let file_name = &hash[2..];

        let dir_path = format!(".git/objects/{}", dir_name);
        let file_path = format!("{}/{}", dir_path, file_name);
        fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;

        let blob_object_file = fs::File::create(file_path).map_err(|e| e.to_string())?;

        let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
        encoder.write_all(sha_input.as_bytes()).map_err(|e| e.to_string())?;
        encoder.finish().map_err(|e| e.to_string())?;

        Ok(())
    }
}