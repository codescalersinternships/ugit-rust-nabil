use std::{collections::{HashMap, HashSet, VecDeque}, fs, io::{self, Error, ErrorKind, Write}, path::Path};

use crate::data::{self, hash_object, update_ref, RefValue};

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
        let entry_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to convert entry name to string")),
        };
        entries.push((entry_name, oid, entry_type));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut tree = String::new();
    for entry in entries {
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



pub fn commit(msg: &str) -> Result<String, io::Error> {
    let tree = write_tree(".")?;
    let mut commit_str =  format!("tree {} \n",tree);
    let head = match data::get_ref("HEAD", true)?.value{
        Some(v) => v,
        None => return Err(Error::new(ErrorKind::InvalidData, format!("refvalue doesn't contain valid value"))),
    };
    //println!("head is{}",head);
    if !head.is_empty() {
        commit_str.push_str(&format!("parent {}\n",head));
    }
    commit_str.push_str("\n");
    commit_str.push_str(msg);
    commit_str.push_str("\n");

    let oid = data::hash_object(&commit_str.into_bytes(), "commit")?;
    data::update_ref("HEAD", &RefValue{symbolic:None, value: Some(oid.clone())}, true )?;
    return Ok(oid);
}


pub fn get_commit(oid: &str) -> Result<(String, String, String), io::Error>  {
    let comit = data::get_object(&oid, "commit")?;
    let mut parent = String::new();
    let mut tree = String::new();
    let mut message = String::new();
    //println!("hereee\n {}\n here awy b2", comit);
    for entry in comit.lines() {
        let space = match entry.chars().position(|c| c == ' '){
            Some(val) => val,
             None => break,
        };
        let cur_key= &entry[..space];
        let cur_value= &entry[space..];
        if cur_key == "tree" {
            tree = String::from(cur_value);
        }else if cur_key == "parent" {
            parent = String::from(cur_value);
        }else {
            panic!("unkonown key");
        }
        message.push_str(entry);
        message.push_str("\n");
    }
    if message.len() > 0 {
        message.pop();
    }

    //println!("{} {} {}",commit_val[0].0, commit_val[0].1, commit_val[0].2);
    return Ok((tree,parent,message));
}

pub fn checkout(oid: &str) -> Result<(), io::Error>{
    let comit = get_commit(&oid)?;
    read_tree(&comit.1)?;
    println!("read");
    data::update_ref("HEAD", &RefValue { symbolic: None, value: Some(String::from(oid)) }, true)?;
    Ok(())
}

pub fn create_tag(name: &str, oid: &str) -> Result<(), io::Error>{
    data::update_ref(&format!("refs/tags/{name}"),&RefValue { symbolic: None, value: Some(String::from(oid)) }, true)
}

fn get_oid(name_par: &str) -> Result<String, io::Error> {
    let mut name = name_par;
    if name == "@" {
        name = "HEAD";
    }
    // Name is ref
    let refs_to_try = vec![
        format!("{}", name),
        format!("refs/{}", name),
        format!("refs/tags/{}", name),
        format!("refs/heads/{}", name),
    ];

    for r in refs_to_try {
        let ref_ret = match data::get_ref(&r, true)?.value {
            Some(v) => v,
            None => return Err(Error::new(ErrorKind::InvalidData, format!("refvalue doesn't contain valid value"))),
        };
        if ref_ret.len() != 0 {
            return Ok(ref_ret);
        }
    }

    // Name is SHA1
    let is_hex = name.chars().all(|c| c.is_ascii_hexdigit());
    if name.len() == 40 && is_hex {
        return Ok(name.to_string());
    }

    panic!("Unknown name {}", name);
}


pub fn iter_commits_and_parents(oids: VecDeque<String>) -> Result<Vec<String>, io::Error>{
    let mut oids = oids;
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    while !oids.is_empty() {
        let oid = match oids.front(){
            Some(val) => val.clone(),
            None => break
        };
        oids.pop_front();

        if visited.contains(&oid) || oid.is_empty() {
            continue;
        }
        visited.insert(oid.clone());
        result.push(oid.clone());
        
        let comit = match get_commit(&oid.trim()){
            Ok(val) => val,
            Err(_) => continue
        };
        if comit.1.is_empty() {
            continue;
        }
        oids.push_front(comit.1.clone());

    }

    Ok(result)
    
}


pub fn create_branch(name: &str, oid: &str) -> Result<(), io::Error>{
    update_ref(name, &RefValue { symbolic: None, value: Some(String::from(oid)) }, true)
}