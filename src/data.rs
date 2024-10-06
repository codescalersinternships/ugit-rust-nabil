use std::{ fs, io::{self, Error, ErrorKind, Write}, path::Path};

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
    let file_content = fs::read(format!("{}/objects/{}", GIT_DIR, oid.trim()))?;
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

pub struct RefValue {
    pub symbolic: Option<bool>,
    pub value: Option<String>
}

pub fn update_ref(reff: &str, value: &RefValue, deref: bool) -> Result<(), io::Error>{
    let real_reff = _get_ref_internal(reff,deref)?.0;
    assert!(!real_reff.is_empty());
    let ref_path = format!("{}/{}", GIT_DIR,real_reff);
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
    let val = match &value.value {
        Some(v) => {
            let mut ret = String::new();
            if let Some(true) =value.symbolic {
                ret = String::from("ref: ");
            }
            ret += v;
            ret
        },
        None => return Err(Error::new(ErrorKind::InvalidData, format!("refvalue doesn't contain valid value"))),
    };
    match fs::write(&ref_path,&val){
        Ok(_) => return Ok(()),
        Err(err) => return Err(Error::new(ErrorKind::InvalidData, format!("file of head isn't found err: {}",err))),
    }
}

pub fn delete_ref(reff: &str, deref: bool) -> Result<(), io::Error> {
    let real_reff = _get_ref_internal(reff, deref)?.0;
    fs::remove_file(format!("{}/{}", GIT_DIR, real_reff))?;
    Ok(())
}

pub fn get_ref(reff: &str, deref: bool) -> Result<RefValue, io::Error> {
    return Ok(_get_ref_internal(reff, deref)?.1);
}

pub fn iter_refs(prefix: &str, deref: bool) -> Result<Vec<(String, RefValue)>,io::Error> {
    let dir: fs::ReadDir = fs::read_dir(format!("{}/refs/", GIT_DIR))?;
    let mut entries = vec!["HEAD".to_string(), "MERGE_HEAD".to_string()];
    for entry in dir {
        let entry = entry?;
        let entry_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to convert entry name to string")),
        };
        entries.push(entry_name);
    }

    let mut ret = Vec::new();
    for entry in entries {
        if !entry.starts_with(prefix) {
            continue;
        }
        let reff = get_ref(&entry.clone(), deref)?;
        if let Some(_val) = reff.value {
            ret.push(( 
                entry.clone(),
                get_ref(&entry.clone(), deref)?));
        }
        
    }
    Ok(ret)
}


fn _get_ref_internal(reff: &str, deref: bool) -> Result<(String, RefValue), io::Error>{
    let ref_path = format!("{}/{}", GIT_DIR, reff);
    let mut value:String = String::new();
    if Path::new(&ref_path).exists() {
        value = match fs::read(&ref_path) {
            Ok(content) => String::from_utf8(content).unwrap_or_else(|_| String::new()),
            Err(_) => return Ok((String::new(),RefValue { symbolic: None, value: None })),
        };
    }
    let symbolic = value.len() > 4 && value[0..4] == (*"ref:");
    if symbolic {
        if deref {
            return Ok(_get_ref_internal(&value[5..], true)?);
        }
        
    }
    Ok((String::from(reff), RefValue{symbolic: Some(symbolic), value: Some(value)}))
}