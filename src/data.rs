use std::{fs, io::{self, Error, ErrorKind, Write}, path::Path};

use sha1::{Digest, Sha1};



const GIT_DIR: &str = ".ugit";

pub fn init() -> Result<(), io::Error>{
    fs::create_dir_all(GIT_DIR)?;
    fs::create_dir_all(format!("{}/objects", GIT_DIR))?;
    fs::create_dir_all(format!("{}/refs", GIT_DIR))?;
    Ok(())
}

pub fn hash_object(data: &Vec<u8>,  expected: &str) -> Result<String, io::Error>{
    // create a Sha1 object
    let mut hasher = Sha1::new();

    // process input message
    hasher.update(data);

    let result = hex::encode(hasher.finalize());

    let object_path = format!("{}/objects/{}", GIT_DIR, result);
    let path = Path::new(&object_path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut combined_data = Vec::new();
    combined_data.extend_from_slice(expected.as_bytes());
    combined_data.extend_from_slice(b"\0");
    combined_data.extend_from_slice(data);
    let mut file = fs::File::create(path)?;
    file.write_all(&combined_data)?;
    return Ok(result)
}

pub fn get_object(oid: &str, expected: &str) -> Result<String, io::Error>{
    let file_content = fs::read(format!("{}/objects/{}", GIT_DIR, oid))?;

    let null_index = match file_content.iter().position(|&b| b == 0){
        Some(val) => val,
        None => return Err(Error::new(ErrorKind::InvalidData, "Invalid object format: no null separator found")),
    };

    let type_ = String::from_utf8(file_content[..null_index].to_vec()).unwrap();

    let content = String::from_utf8(file_content[null_index + 1..].to_vec()).unwrap();

    if type_ != expected {
        return Err(Error::new(ErrorKind::InvalidData, "types isn't right"));
    }
    return Ok(content)
}

pub fn update_ref(reff: &str, oid: &str) -> Result<(), io::Error>{
    let ref_path = format!("{}/{}", GIT_DIR,reff);
    if !Path::new(&ref_path).exists() {
        let path = std::path::Path::new(&ref_path);
        let prefix = match path.parent(){
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "cant get parent path in update ref")),
        };
        match fs::create_dir_all(prefix){
            Ok(val) => val,
            Err(err) => return Err(Error::new(ErrorKind::InvalidData, format!("couldn't create parent path in update ref: {}",err))),
        };
        match fs::File::create(path){
            Ok(val) => val,
            Err(err) => return Err(Error::new(ErrorKind::InvalidData, format!("couldn't create head file: {}",err))),
        };
    }
    match fs::write(&ref_path,oid){
        Ok(_) => return Ok(()),
        Err(err) => return Err(Error::new(ErrorKind::InvalidData, format!("file of head isn't found err: {}",err))),
    }
}

pub fn get_ref(reff: &str) -> Result<String, io::Error> {
    let ref_path = format!("{}/{}", GIT_DIR, reff);
    if !Path::new(&ref_path).exists() {
        return Ok(String::new());
    }
    match fs::read(&ref_path) {
        Ok(content) => Ok(String::from_utf8(content).unwrap_or_else(|_| String::new())),
        Err(_) => Ok(String::new()),
    }
}