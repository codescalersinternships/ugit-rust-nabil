use crate::base;
use crate::data;
use crate::data::hash_object;
use std::fs;
use std::io;
use std::path::Path;


pub fn write_tree(directory: &str) {
    let dir = fs::read_dir(directory).unwrap();

    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        let full_path = path.to_string_lossy().to_string();

        if path.is_file() {
            if is_ignored(&full_path) {
                continue;
            }
            let data = match  fs::read(full_path.clone()){
                Ok(val) => val,
                Err(err) => return, 
            };
            let newdata = data::hash_object(&data);
            println!("{}", full_path);
        } else if path.is_dir() {
            write_tree(&full_path);
        }
    }

    // TODO: Actually create the tree object
}

pub fn is_ignored(directory: &str) -> bool{
    if directory.len() < 5 {
        return false;
    }
    if &directory[..5] == ".ugit" {
        return true;
    }
    return false;
}