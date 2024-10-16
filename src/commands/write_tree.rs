use std::fs;
use sha1::{Digest, Sha1};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

pub struct WriteTree;

impl WriteTree {
    pub fn run(_args: &[String]) -> Result<(), String> {
        let tree_hash = Self::write_tree(".")?;
        print!("{}", tree_hash);
        Ok(())
    }

    fn write_tree(path: &str) -> Result<String, String> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name().into_string().map_err(|_| "Invalid file name".to_string())?;
            if file_name == ".git" { continue; }
            
            let file_type = entry.file_type().map_err(|e| e.to_string())?;
            let mode = if file_type.is_dir() { "40000" } else { "100644" };
            
            let raw_hash = if file_type.is_dir() {
                let subtree_hash = Self::write_tree(&entry.path().to_str().ok_or("Invalid path")?)?;
                hex::decode(subtree_hash).map_err(|e| e.to_string())?
            } else {
                let contents = fs::read(entry.path()).map_err(|e| e.to_string())?;
                let (_, raw_hash) = Self::hash_object("blob", &contents);
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
        
        Self::write_object(path, "tree", &tree_content)
    }

    fn hash_object(object_type: &str, contents: &[u8]) -> (String, Vec<u8>) {
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

    fn write_object(path: &str, object_type: &str, contents: &[u8]) -> Result<String, String> {
        let (hash, _) = Self::hash_object(object_type, contents);

        let dir_name = &hash[..2];
        let file_name = &hash[2..];
        let dir_path = format!("{}/.git/objects/{}", path, dir_name);
        let file_path = format!("{}/{}", dir_path, file_name);

        fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;

        let size = contents.len();
        let mut object_content = Vec::new();
        object_content.extend_from_slice(object_type.as_bytes());
        object_content.extend_from_slice(b" ");
        object_content.extend_from_slice(size.to_string().as_bytes());
        object_content.extend_from_slice(b"\0");
        object_content.extend_from_slice(contents);

        let blob_object_file = fs::File::create(file_path).map_err(|e| e.to_string())?;
        let mut encoder = ZlibEncoder::new(blob_object_file, Compression::default());
        encoder.write_all(&object_content).map_err(|e| e.to_string())?;
        encoder.finish().map_err(|e| e.to_string())?;

        Ok(hash)
    }
}