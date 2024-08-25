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
        data::cat_file(&args[2])
    }else if args[1] == "write-tree" {
        println!("{}",args[2]);
        base::write_tree(&args[2])
    }
    
}


fn hash_object(file_path: &str) {
    let data = match  fs::read(file_path){
        Ok(val) => val,
        Err(err) => return, 
    };
    let newdata = data::hash_object(&data);
    println!("Object ID: {}", newdata);

}