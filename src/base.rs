use std::{collections::HashMap, fs, io::{self, Error, ErrorKind, Write}, path::Path};

use crate::data::{self, hash_object};

pub fn empty_current_directory() -> Result<(), io::Error> {
    let directory = Path::new(".");
    let dir = fs::read_dir(directory)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let full_path = match path.to_str() {
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "Failed to convert path to string")),
        };
        if is_ignored(&full_path) {
            continue;
        }
        //println!("{}",full_path);
        if path.is_file() {
            match fs::remove_file(&path){
                Ok(_) => continue,
                Err(err) => return Err(Error::new(ErrorKind::InvalidData,format!("file remove error: {}", err))),
            };
        } else if path.is_dir() {
            match fs::remove_dir_all(&path){
                Ok(_) => continue,
                Err(err) => return Err(Error::new(ErrorKind::InvalidData,format!("directory remove error: {}", err))),
            };
        }
    }
    Ok(())
}


pub fn write_tree(directory: &str) -> Result<String, io::Error>{
    let dir: fs::ReadDir = fs::read_dir(directory)?;
    let mut entries = Vec::new();

    for entry in dir {
        let entry = entry?;
        let path = entry.path();

        let full_path = match path.to_str() {
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "Failed to convert path to string")),
        };
        
        let entry_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to convert entry name to string")),
        };

        if is_ignored(&full_path) {
            continue;
        }
        //println!("{}",full_path);
        let (oid, entry_type) = if path.is_file() {
            let data = fs::read(full_path)?;
            let newdata = data::hash_object(&data,"blob")?;
            (newdata,"blob")
        } else if path.is_dir() {
            (write_tree(&full_path)?,"tree")
        }else {
            continue;
        };
        entries.push((entry_name, oid, entry_type));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut tree:String = String::new();
    for entry in &entries {
        let s = format!("{} {} {}\n", entry.2, entry.1, entry.0);
        tree.push_str(&s);
    }
    return  hash_object(&tree.into_bytes(),"tree");
}

pub fn is_ignored(directory: &str) -> bool{
    return  directory.contains(".ugit") || directory.contains(".git");
}


fn iter_tree_entries(oid: &str) -> Result<Vec<(String, String, String)>, io::Error> {
    if oid.is_empty() {
        return Ok(Vec::new());
    }

    let tree = data::get_object(&oid.to_string(), "tree")?;
    let mut ret: Vec<(String, String, String)> = Vec::new();

    for entry in tree.lines() {
        let parts: Vec<&str> = entry.splitn(3, ' ').collect();
        ret.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ));
    }

    Ok(ret)
}


fn get_tree(oid: &str, base_path: &str) -> Result< HashMap<String, String>, io::Error> {
    let mut result: HashMap<String, String> = HashMap::new();

    for (type_, oid, name) in iter_tree_entries(oid)? {
        assert!(!name.contains('/'));
        assert!(name != ".." && name != ".");

        let path = format!("{}{}", base_path, name);
        if type_ == "blob" {
            result.insert(path, oid);
        } else if type_ == "tree" {
            let subtree = get_tree(&oid, &format!("{}/", path))?;
            result.extend(subtree);
        } else {
            return Err(Error::new(ErrorKind::InvalidData, format!("Unknown tree entry {}", type_)));
        }
    }

    return Ok(result);
}

pub fn read_tree(tree_oid: &str) -> Result<(), io::Error> {
    //empty_current_directory()?;
    let tree = get_tree(tree_oid, ".ugit/objects/")?;

    for (path, oid) in tree {
        let path_dir = match Path::new(&path).parent(){
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "Failed to get path parent")),
        };
        fs::create_dir_all(path_dir)?;

        let object_data = data::get_object(&oid, "blob")?;
        let mut file = fs::File::create(&path)?;
        file.write_all(&object_data.as_bytes())?;
    }
    Ok(())
}