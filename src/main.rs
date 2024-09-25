use std::{env, fs::{self}, io::{self}};

mod data;
mod base;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("args aren't enough")
    }
    if args[1] == "init" {
        data::init().unwrap();
        let path = match env::current_dir(){
            Ok(path)=> path,
            Err(_) => return,
        };
        println!("Initialized empty ugit repository in {}",path.display());
    }else if args[1] == "hash-object" {
        hash_object(&args[2]).unwrap();
    }else if args[1] == "cat-file" {
        println!("{}",data::get_object(&args[2],"tree").unwrap())
    }else if args[1] == "write-tree" {
        println!("{}",base::write_tree(&args[2]).unwrap())
    }else if args[1] == "read-tree" {
        base::read_tree(&args[2]).unwrap()
    }
    
}


fn hash_object(file_path: &str) -> Result<(), io::Error> {
    let data = fs::read(file_path)?;
    let newdata = data::hash_object(&data,"blob")?;
    println!("Object ID: {}", newdata);
    Ok(())
}