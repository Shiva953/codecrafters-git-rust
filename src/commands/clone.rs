use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use flate2::read::ZlibDecoder;
use reqwest;
use sha1::{Sha1, Digest};

pub struct Clone;

impl Clone {
    pub fn run(args: &[String]) -> Result<(), String> {
        if args.len() != 2 {
            return Err("Usage: git clone <repo_url> <target_directory>".to_string());
        }

        let repo_url = &args[0];
        let target_dir = Path::new(&args[1]);

        println!("Cloning repository: {}", repo_url);
        println!("Target directory: {}", target_dir.display());

        // Create target directory
        fs::create_dir_all(target_dir).map_err(|e| format!("Failed to create target directory: {}", e))?;

        // Initialize Git repository
        Self::init_repository(target_dir)?;

        // Discover references
        let refs = Self::discover_refs(repo_url)?;
        println!("Discovered {} refs", refs.len());

        // Get packfile
        let packfile = Self::fetch_packfile(repo_url, &refs)?;
        println!("Fetched packfile of size: {} bytes", packfile.len());

        // Process packfile
        Self::process_packfile(&packfile, target_dir)?;

        // Update refs
        Self::update_refs(target_dir, &refs)?;

        println!("Repository cloned successfully.");
        Ok(())
    }

    fn init_repository(target_dir: &Path) -> Result<(), String> {
        let git_dir = target_dir.join(".git");
        fs::create_dir_all(&git_dir).map_err(|e| format!("Failed to create .git directory: {}", e))?;
        
        for dir in &["objects", "refs", "refs/heads"] {
            fs::create_dir_all(git_dir.join(dir))
                .map_err(|e| format!("Failed to create {} directory: {}", dir, e))?;
        }

        fs::write(git_dir.join("HEAD"), "ref: refs/heads/master\n")
            .map_err(|e| format!("Failed to create HEAD file: {}", e))?;

        Ok(())
    }

    fn discover_refs(repo_url: &str) -> Result<Vec<(String, String)>, String> {
        let url = format!("{}/info/refs?service=git-upload-pack", repo_url);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| format!("Failed to fetch refs: {}", e))?
            .text()
            .map_err(|e| format!("Failed to read refs response: {}", e))?;

        println!("Refs response:\n{}", response);

        let mut refs = Vec::new();
        for line in response.lines().skip(2) {
            let line = line.trim_start_matches(|c: char| c.is_ascii_hexdigit() || c == ' ');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                refs.push((parts[1].to_string(), parts[0].to_string()));
            }
        }

        if refs.is_empty() {
            return Err("No valid refs found in the response".to_string());
        }

        Ok(refs)
    }

    fn fetch_packfile(repo_url: &str, refs: &[(String, String)]) -> Result<Vec<u8>, String> {
        let url = format!("{}/git-upload-pack", repo_url);
        let want_ref = &refs[0].1; // Use the first ref as the one we want

        let body = format!("0032want {}\n00000009done\n", want_ref);

        println!("Sending request to: {}", url);
        println!("Request body: {:?}", body);

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .body(body)
            .header("Content-Type", "application/x-git-upload-pack-request")
            .send()
            .map_err(|e| format!("Failed to fetch packfile: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Server returned error status: {}", response.status()));
        }

        let content = response.bytes().map_err(|e| format!("Failed to read packfile: {}", e))?.to_vec();
        
        if content.is_empty() {
            return Err("Received empty packfile".to_string());
        }

        println!("Received response of size: {} bytes", content.len());
        println!("First few bytes: {:?}", &content[..std::cmp::min(content.len(), 20)]);

        Ok(content)
    }

    fn process_packfile(packfile: &[u8], target_dir: &Path) -> Result<(), String> {
        if packfile.len() < 12 {
            return Err(format!("Packfile too short: {} bytes", packfile.len()));
        }

        let mut reader = Cursor::new(packfile);
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature).map_err(|e| format!("Failed to read packfile signature: {}", e))?;
        
        println!("Packfile signature: {:?}", signature);
        
        if &signature != b"PACK" {
            return Err(format!("Invalid packfile signature: {:?}", signature));
        }

        let mut version = [0u8; 4];
        reader.read_exact(&mut version).map_err(|e| format!("Failed to read packfile version: {}", e))?;
        let version = u32::from_be_bytes(version);
        
        println!("Packfile version: {}", version);
        
        if version != 2 {
            return Err(format!("Unsupported packfile version: {}", version));
        }

        let mut object_count = [0u8; 4];
        reader.read_exact(&mut object_count).map_err(|e| format!("Failed to read object count: {}", e))?;
        let object_count = u32::from_be_bytes(object_count);

        println!("Processing packfile with {} objects", object_count);

        for i in 0..object_count {
            Self::extract_object(&mut reader, target_dir)
                .map_err(|e| format!("Failed to extract object {}/{}: {}", i + 1, object_count, e))?;
        }

        println!("Finished processing packfile");
        Ok(())
    }

    fn extract_object<R: Read>(reader: &mut R, target_dir: &Path) -> Result<(), String> {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).map_err(|e| format!("Failed to read object type: {}", e))?;
        let obj_type = (byte[0] & 0x70) >> 4;
        let mut size = (byte[0] & 0x0F) as u64;
        let mut shift = 4;

        loop {
            reader.read_exact(&mut byte).map_err(|e| format!("Failed to read object size: {}", e))?;
            size |= ((byte[0] & 0x7F) as u64) << shift;
            shift += 7;
            if byte[0] & 0x80 == 0 {
                break;
            }
        }

        let mut decoder = ZlibDecoder::new(reader);
        let mut content = Vec::new();
        decoder.read_to_end(&mut content).map_err(|e| format!("Failed to decompress object: {}", e))?;

        let hash = Sha1::digest(&content);
        let hash_str = format!("{:x}", hash);
        let obj_path = target_dir.join(".git/objects").join(&hash_str[..2]).join(&hash_str[2..]);

        fs::create_dir_all(obj_path.parent().unwrap()).map_err(|e| format!("Failed to create object directory: {}", e))?;
        fs::write(&obj_path, content).map_err(|e| format!("Failed to write object file: {}", e))?;

        println!("Extracted object: {}", hash_str);
        Ok(())
    }

    fn update_refs(target_dir: &Path, refs: &[(String, String)]) -> Result<(), String> {
        let git_dir = target_dir.join(".git");

        for (name, sha) in refs {
            if name.starts_with("refs/") {
                let ref_file = git_dir.join(name);
                fs::create_dir_all(ref_file.parent().unwrap()).map_err(|e| format!("Failed to create ref directory: {}", e))?;
                fs::write(ref_file, format!("{}\n", sha)).map_err(|e| format!("Failed to write ref {}: {}", name, e))?;
            }
        }

        // Update HEAD
        let head_ref = refs.iter().find(|(name, _)| name == "HEAD" || name.ends_with("/HEAD"));
        if let Some((_, sha)) = head_ref {
            fs::write(git_dir.join("HEAD"), format!("ref: refs/heads/master\n")).map_err(|e| format!("Failed to write HEAD: {}", e))?;
        }

        Ok(())
    }
}