use std::env;
use std::fs;
use std::io;

mod data;
mod base;

fn main() {
    let args: Vec<String> = env::args().collect();
    //dbg!(args);
    //println!("{}",args[1]);
    if args[1] == "init" {
        data::init();
        let path = match env::current_dir(){
            Ok(path)=> path,
            Err(_) => return,
        };
        println!("Initialized empty ugit repository in {}/",path.display());
    }else if args[1] == "hash-object" {
        hash_object(&args[2]);
    }else if args[1] == "cat-file" {
        println!("{}",data::get_object(&args[2],"tree"))
    }else if args[1] == "write-tree" {
        println!("{}",base::write_tree(&args[2]))
    }else if args[1] == "read-tree" {
        base::read_tree(&args[2])
    }else if args[1] == "commit" {
        println!("{}",base::commit(&args[2]))
    }else if args[1] == "log" {
        log("".to_string());
    }else if args[1] == "checkout" {
        base::checkout(args[2].clone());
    }
    
}


fn hash_object(file_path: &str) {
    let data = match  fs::read(file_path){
        Ok(val) => val,
        Err(err) => return, 
    };
    let newdata = data::hash_object(&data,"blob");
    println!("Object ID: {}", newdata);

}

fn log(text: String) {
    let mut oid: String = data::get_head();
    if text.len() > 0 {
        oid = text.clone();
    }
    while oid.len() != 0 {
        let commit = base::get_commit(oid.clone());
        print!("commit {}\n",oid.clone());
        println!("{}",commit[0].2);
        oid = commit[0].1.clone();
    }
}