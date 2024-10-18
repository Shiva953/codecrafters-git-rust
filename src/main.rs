mod commands;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: <command> [<args>]");
        return;
    }

    let result = match args[1].as_str() {
        "init" => commands::init::Init::run(&args[2..]),
        "cat-file" => commands::cat_file::CatFile::run(&args[2..]),
        "hash-object" => commands::hash_object::HashObject::run(&args[2..]),
        "ls-tree" => commands::ls_tree::LsTree::run(&args[2..]),
        "write-tree" => commands::write_tree::WriteTree::run(&args[2..]),
        "clone" => commands::clone::Clone::run(&args[2..]),
        _ => Err(format!("Unknown command: {}", args[1])),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}