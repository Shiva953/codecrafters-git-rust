use std::fs;
use flate2::read::ZlibDecoder;
use std::io::prelude::*;

pub struct LsTree;

impl LsTree {
    pub fn run(args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("Usage: ls-tree <tree_sha>".to_string());
        }
        let tree_sha = &args[0];
        let path = format!(".git/objects/{}/{}", &tree_sha[..2], &tree_sha[2..]);
        let content = fs::read(path).map_err(|e| e.to_string())?;
        let mut decompressed_data = ZlibDecoder::new(&content[..]);
        let mut tree_file_contents_vec = Vec::new();
        decompressed_data.read_to_end(&mut tree_file_contents_vec)
            .map_err(|e| e.to_string())?;

        let entries = Self::parse_tree_object(&tree_file_contents_vec);

        for name in entries {
            println!("{}", name);
        }
        Ok(())
    }

    fn parse_tree_object(content: &[u8]) -> Vec<String> {
        let mut entries = Vec::new();
        let mut i = 0;

        while i < content.len() && content[i] != 0 {
            i += 1;
        }
        i += 1;

        while i < content.len() {
            let mut name = Vec::new();
            while i < content.len() && content[i] != b' ' {
                i += 1;
            }
            i += 1;
            while i < content.len() && content[i] != 0 {
                name.push(content[i]);
                i += 1;
            }
            i += 1;
            i += 20;

            if let Ok(name_str) = String::from_utf8(name) {
                entries.push(name_str);
            }
        }

        entries
    }
}