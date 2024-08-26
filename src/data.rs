use std::fs;
use std::io;
use std::io::Write;
use std::vec;
use hex_literal::hex;
use sha1::{Sha1, Digest};
use crate::data;
use std::path::Path;

const GIT_DIR: &str = ".ugit";

pub fn init() {
    fs::create_dir_all(GIT_DIR).unwrap();
    fs::create_dir_all(format!("{}/objects", GIT_DIR)).unwrap();
    fs::create_dir_all(format!("{}/refs", GIT_DIR)).unwrap();
    //fs::write(format!("{}/HEAD", GIT_DIR), "ref:refs/heads/master").unwrap();
}

pub fn hash_object(data: &Vec<u8>,  expected: &str) -> String{
    // create a Sha1 object
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(data);

    let result: String = format!("{:X}", hasher.finalize());

    let object_path = format!("{}/objects/{}", GIT_DIR, result);
    let path = Path::new(&object_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }
    let mut data = data.clone();
    let expected_bytes = expected.as_bytes();
    data = [expected_bytes,b"\0", &data].concat();
    let mut file = fs::File::create(path).expect("Failed to create file");
    match file.write_all(&data){
        Ok(_) => println!("file wrote succesfully"),
        Err(_) => println!("error"),
    };
    return result;
}


pub fn get_object(path: &String, expected: &str) -> String{
    let mut content = Vec::new();
    content = fs::read(format!("{}/objects/{}", GIT_DIR, path)).unwrap();

    // Find the first null byte to separate the type and content
    let null_index = content.iter().position(|&b| b == 0)
        .expect("Invalid object format: no null separator found");

    let type_ = String::from_utf8(content[..null_index].to_vec()).unwrap();

    let ret = String::from_utf8(content[null_index + 1..].to_vec()).unwrap();

    if type_ != expected {
        panic!("types isn't right")
    }
    return ret;
}