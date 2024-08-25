use std::fs;
use std::io;
use std::io::Write;
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

pub fn hash_object(data: &Vec<u8>) -> String{
    // create a Sha1 object
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(data);

    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    // let result = hasher.finalize();
    // assert_eq!(result[..], hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"));
    let result: String = format!("{:X}", hasher.finalize());

    let object_path = format!("{}/objects/{}", GIT_DIR, result);
    let path = Path::new(&object_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = fs::File::create(path).expect("Failed to create file");
    match file.write_all(&data){
        Ok(_) => println!("file wrote succesfully"),
        Err(_) => println!("error"),
    };
    return result;
}


pub fn cat_file(path: &String) {
    let content = fs::read(format!(".ugit/objects/{}", path)).unwrap();
        let s = String::from_utf8_lossy(&content);
        println!("{}", s);
}