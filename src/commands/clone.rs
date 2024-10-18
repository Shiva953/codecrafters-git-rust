use std::fs;
use std::path::Path;
use reqwest;
use flate2::read::ZlibDecoder;
use std::io::Read;
use sha1::{Sha1, Digest};

pub struct Clone;

impl Clone {
    pub fn run(args: &[String]) -> Result<(), String> {
        if args.len() != 2 {
            return Err("Usage: git clone <repo_url> <target_directory>".to_string());
        }

        let repo_url = &args[0];
        let target_dir = &args[1];

        fs::create_dir_all(target_dir).map_err(|e| format!("Failed to create target directory: {}", e))?;

        let refs = Self::discover_refs(repo_url)?;
        let packfile = Self::fetch_packfile(repo_url, &refs)?;
        Self::process_packfile(&packfile, target_dir)?;
        Self::update_refs(target_dir, &refs)?;

        Ok(())
    }

    fn discover_refs(repo_url: &str) -> Result<Vec<(String, String)>, String> {
        let url = format!("{}/info/refs?service=git-upload-pack", repo_url);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| format!("Failed to fetch refs: {}", e))?
            .text()
            .map_err(|e| format!("Failed to read refs response: {}", e))?;

        // Parse the response to extract refs
        let refs: Vec<(String, String)> = response
            .lines()
            .filter(|line| line.contains("refs/"))
            .map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                (parts[1].to_string(), parts[0].to_string())
            })
            .collect();

        Ok(refs)
    }

    fn fetch_packfile(repo_url: &str, refs: &[(String, String)]) -> Result<Vec<u8>, String> {
        let url = format!("{}/git-upload-pack", repo_url);
        let want_ref = &refs[0].1; // Use the first ref as the one we want

        let body = format!("0032want {}\n00000009done\n", want_ref);

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .body(body)
            .header("Content-Type", "application/x-git-upload-pack-request")
            .send()
            .map_err(|e| format!("Failed to fetch packfile: {}", e))?;

        response.bytes().map_err(|e| format!("Failed to read packfile: {}", e)).map(|b| b.to_vec())
    }

    fn process_packfile(packfile: &[u8], target_dir: &str) -> Result<(), String> {
        let mut decoder = ZlibDecoder::new(packfile);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress packfile: {}", e))?;

        Ok(())
    }

    fn update_refs(target_dir: &str, refs: &[(String, String)]) -> Result<(), String> {
        let git_dir = Path::new(target_dir).join(".git");
        fs::create_dir_all(git_dir.join("refs/heads")).map_err(|e| format!("Failed to create refs directory: {}", e))?;

        for (name, sha) in refs {
            if name.starts_with("refs/heads/") {
                let ref_file = git_dir.join(name);
                fs::write(ref_file, sha).map_err(|e| format!("Failed to write ref {}: {}", name, e))?;
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