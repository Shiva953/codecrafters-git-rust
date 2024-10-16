use std::fs;

pub struct Init;

impl Init {
    pub fn run(_args: &[String]) -> Result<(), String> {
        fs::create_dir(".git").map_err(|e| e.to_string())?;
        fs::create_dir(".git/objects").map_err(|e| e.to_string())?;
        fs::create_dir(".git/refs").map_err(|e| e.to_string())?;
        fs::write(".git/HEAD", "ref: refs/heads/main\n").map_err(|e| e.to_string())?;
        println!("Initialized git directory");
        Ok(())
    }
}