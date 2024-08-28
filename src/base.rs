use crate::base;
use crate::data;
use crate::data::hash_object;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use std::vec;


pub fn empty_current_directory() {
    let directory = Path::new(".");
    let dir = fs::read_dir(directory).unwrap();

    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        let full_path = path.to_string_lossy().to_string();
        if is_ignored(&full_path) {
            continue;
        }
        //println!("{}",full_path);
        if path.is_file() {
            
            fs::remove_file(&path).expect("file remove error")
        } else if path.is_dir() {
            fs::remove_dir(&path).expect("directory remove error")
        }
    }

}

pub fn write_tree(directory: &str) -> String{
    let dir = fs::read_dir(directory).unwrap();
    let mut entries = Vec::new();

    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        let full_path = path.to_string_lossy().to_string();
        let entry_name = entry.file_name().into_string().unwrap_or_default();
        if is_ignored(&full_path) {
            continue;
        }
        //println!("{}",full_path);
        let (oid, entry_type) = if path.is_file() {
            
            let data = fs::read(full_path.clone()).unwrap();
            let newdata = data::hash_object(&data,"blob");
            (newdata,"blob")
        } else if path.is_dir() {
            (write_tree(&full_path),"tree")
        }else {
            continue;
        };
        entries.push((entry_name, oid.clone(), entry_type.clone()));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut tree:String = String::new();
    let mut idx = 0 ;
    while idx < entries.len() {
        let s = format!("{} {} {}\n", entries[idx].2,entries[idx].1, entries[idx].0);
        tree.push_str(&s);
        idx = idx +1;
    }
    return  hash_object(&tree.into_bytes(),"tree");
}

pub fn is_ignored(directory: &str) -> bool{
    return  directory.contains(".ugit") || directory.contains(".git");
}

fn iter_tree_entries(oid: &str) -> Vec<(String, String, String)> {
    if oid.is_empty() {
        return Vec::new();
    }

    let tree = data::get_object(&oid.to_string(), "tree");
    let mut ret: Vec<(String, String, String)> = Vec::new();

    for entry in tree.lines() {
        let parts: Vec<&str> = entry.splitn(3, ' ').collect();
        ret.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ));
    }

    ret
}


fn get_tree(oid: &str, base_path: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();

    for (type_, oid, name) in iter_tree_entries(oid) {
        assert!(!name.contains('/'));
        assert!(name != ".." && name != ".");

        let path = format!("{}{}", base_path, name);
        if type_ == "blob" {
            result.insert(path, oid);
        } else if type_ == "tree" {
            let subtree = get_tree(&oid, &format!("{}/", path));
            result.extend(subtree);
        } else {
            panic!("Unknown tree entry {}", type_);
        }
    }

    return result;
}

pub fn read_tree(tree_oid: &str) {
    let tree = get_tree(tree_oid, ".ugit/objects/");

    for (path, oid) in tree {
        let path_dir = Path::new(&path).parent().unwrap();
        fs::create_dir_all(path_dir).expect("Failed to create directory");

        let object_data = data::get_object(&oid, "blob");
        let mut file = fs::File::create(&path).expect("Failed to create file");
        file.write_all(&object_data.as_bytes()).expect("Failed to write to file");
    }
}

pub fn commit(msg: &String) -> String {
    let tree = write_tree(".");
    let mut commitStr: String =  format!("tree {} \n",tree);
    let HEAD = data::get_head();
    if HEAD != "" {
        commitStr += format!("parent {HEAD}\n").as_str();
    }
    commitStr += "\n";
    commitStr += format!("{msg}\n").as_str();

    let oid = data::hash_object(&commitStr.into_bytes(), "commit");
    data::set_head(&oid);
    return oid;
}


pub fn get_commit(oid: String) -> Vec<(String, String, String)>  {
    let comit = data::get_object(&oid, "commit");
    let mut parent: String = "".to_string();
    let mut tree: String = "".to_string();
    let mut message: String = "".to_string();
    for entry in comit.lines() {
        let space = entry.chars().position(|c| c == ' ')
        .expect("Invalid object format: no space separator found");
        let cur_key:String = entry[..space].to_string();
        let cur_value:String = entry[space..].to_string();
        if cur_key == "tree" {
            tree = cur_value;
        }else if cur_key == "parent" {
            tree = cur_value;
        }else {
            panic!("unkonown key");
        }
        message += entry;
        message += "\n";
    }
    if message.len() > 0 {
        message.pop();
    }

    let mut ret:Vec<(String, String, String)> = Vec::new();
    ret.push((tree,parent,message));
    return ret;
}

pub fn checkout(oid: String) {
    let comit = get_commit(oid.clone());
    read_tree(&comit[0].1);
    data::set_head(&oid);
}