use std::{collections::{vec_deque, HashMap, HashSet, VecDeque}, fs::{self, File}, io::{self, Error, ErrorKind, Read, Write}, path::{Path, PathBuf}, vec};

use walkdir::WalkDir;

use crate::{data::{self, get_ref, hash_object, iter_refs, update_ref, RefValue}, diff};

pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub msg: String,
}

pub fn init() -> Result<(), io::Error>{
    data::init()?;
    update_ref("HEAD", &RefValue { symbolic: Some(true), value: Some(String::from("refs/heads/master")) }, true)?;
    Ok(())
}
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


pub fn get_tree(oid: &str, base_path: &str) -> Result< HashMap<String, String>, io::Error> {
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
    if let Some(MERGE_HEAD) = data::get_ref("MERGE_HEAD", true)?.value {
        commit_str.push_str(&format!("parent {MERGE_HEAD}\n"));
        data::delete_ref("MERGE_HEAD", false);
    }
    commit_str.push_str("\n");
    commit_str.push_str(msg);
    commit_str.push_str("\n");

    let oid = data::hash_object(&commit_str.into_bytes(), "commit")?;
    data::update_ref("HEAD", &RefValue{symbolic:None, value: Some(oid.clone())}, true )?;
    return Ok(oid);
}


pub fn get_commit(oid: &str) -> Result<Commit, io::Error>  {
    let comit = data::get_object(&oid, "commit")?;
    let mut parents = Vec::new();
    let mut tree = String::new();
    let mut message = String::new();
    //println!("hereee\n {}\n here awy b2", comit);
    for entry in comit.lines() {
        println!("inn{entry}");
        if let Some(space) = entry.chars().position(|c| c == ' ') {
            let cur_key= &entry[..space];
            let cur_value= &entry[space..];
            if cur_key == "tree" {
                tree = String::from(cur_value);
            }else if cur_key == "parent" {
                parents.push(String::from(cur_value));
            }else {
                panic!("unkonown key");
            }
            continue;
        }
        
        message.push_str(entry);
        message.push_str("\n");
    }
    if message.len() > 0 {
        message.pop();
    }

    //println!("{} {} {}",commit_val[0].0, commit_val[0].1, commit_val[0].2);
    return Ok(Commit { tree: tree, parents: parents, msg: message });
}

pub fn checkout(name: &str) -> Result<(), io::Error>{
    let oid = get_oid(name)?;
    let comit = get_commit(&name)?;
    read_tree(&comit.parents[0])?;
    let HEAD: RefValue;
    if is_branch(name)? {
        HEAD = RefValue{symbolic: Some(true), value: Some(format!("refs/heads/{name}"))}
    }else {
        HEAD = RefValue{symbolic: Some(false), value: Some(oid)}
    }
    data::update_ref("HEAD", &HEAD, false)?;
    Ok(())
}

fn is_branch(name: &str) -> Result<bool, io::Error> {
    let _reff = match get_ref(&format!("refs/heads/{name}"), true)?.value{
        Some(_val) => return Ok(true),
        None => return Ok(false)
    };
}

pub fn create_tag(name: &str, oid: &str) -> Result<(), io::Error>{
    data::update_ref(&format!("refs/tags/{name}"),&RefValue { symbolic: None, value: Some(String::from(oid)) }, true)
}

pub fn get_oid(name_par: &str) -> Result<String, io::Error> {
    let mut name = name_par;
    if name == "@" {
        name = "master";
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
    while let Some(oid) = oids.pop_front() {
        if oid.is_empty() || !visited.insert(oid.clone()) {
            continue;
        }
        result.push(oid.clone());
        
        let comit = match get_commit(&oid.trim()){
            Ok(val) => val,
            Err(_) => continue
        };
        if let Some(first_parent) = comit.parents.get(0) {
            oids.push_front(first_parent.clone());
        }

        // Push the rest of the parents (if any) to the back of the deque
        for parent in comit.parents.iter().skip(1) {
            oids.push_back(parent.clone());
        }
    }

    Ok(result)
    
}


pub fn create_branch(name: &str, oid: &str) -> Result<(), io::Error>{
    update_ref(name, &RefValue { symbolic: None, value: Some(String::from(oid)) }, true)
}

pub fn get_branch_name() -> Result<String, io::Error> {
    let HEAD = get_ref("HEAD", false)?;
    if let None = HEAD.symbolic {
        return Ok(String::new());
    }
    let head = match HEAD.value{
        Some(val) => val,
        None => return Err(Error::new(ErrorKind::InvalidData, format!("refvalue doesn't contain valid value"))),
    };
    println!("{head}");
    if !head[5..].starts_with("refs/heads/") {
        panic!("head doesn't start with refs/heads/");
    }
    Ok(relpath(&head, "refs/heads/")?)
}

fn relpath(refname: &str, base: &str) -> Result<String, io::Error> {
    let ref_path = Path::new(&refname[5..]);
    let base_path = Path::new(base);
    match ref_path.strip_prefix(base_path) {
        Ok(relative_path) => Ok(relative_path.to_string_lossy().into_owned()),
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, format!("refname isn't under base"))),
    }
    
}

pub fn iter_branch_names() -> Result<Vec<String>, io::Error> {
    let mut branches = Vec::new();
    for (refname, _) in iter_refs("refs/heads/", true)? {
        branches.push(relpath(&refname, "refs/heads/")?);
    }
    Ok(branches)
}

pub fn reset(oid: &str) -> Result<(), io::Error> {
    update_ref("HEAD", &RefValue { symbolic: Some(false), value: Some(String::from(oid)) }, true)
}


pub fn get_working_tree() -> io::Result<HashMap<String, String>> {
    let mut result = HashMap::new();

    for entry in WalkDir::new(".") {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let rel_path = path.strip_prefix(".").unwrap_or(path);
            let rel_path_str = rel_path.to_string_lossy().to_string();

            if is_ignored(&rel_path_str) {
                continue;
            }

            let mut file = fs::File::open(path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            let hash = hash_object(&contents, "blob")?;
            result.insert(rel_path_str, hash);
        }
    }

    Ok(result)
}

fn read_tree_merged(t_base: &str, t_head: &str, t_other: &str) -> Result<(), io::Error> { 

    empty_current_directory()?;
    for (path, blob) in diff::merge_trees(get_tree(t_base, ".")?, get_tree(t_head, ".")?, get_tree(t_other, ".")?)? {
        let path_buf = Path::new(&path).parent().unwrap();
        fs::create_dir_all(path_buf)?;
        let mut file = File::create(&path)?;
        file.write_all(&blob)?;

    }

    Ok(())
}

pub fn merge(other: &str) -> Result<(), io::Error> {
    let head_ref = data::get_ref("HEAD", true)?;
    let head = match head_ref.value{
        Some(val) => val,
        None => return Err(Error::new(ErrorKind::InvalidData, format!("refvalue doesn't contain valid value")))
    };

    let merge_base = get_merge_base(other, &head)?;
    let commit_other = get_commit(other)?;

    // Handle fast-forward merge
    if merge_base == head {
        read_tree(&commit_other.tree)?;
        data::update_ref("HEAD", &data::RefValue { symbolic: Some(false), value: Some(String::from(other) ) }, true)?;
        println!("Fast-forward merge, no need to commit");
        return Ok(());
    }

    data::update_ref("MERGE_HEAD", &data::RefValue { symbolic: Some(false), value: Some(String::from(other) ) }, true)?;

    let commit_base = get_commit(&merge_base)?;
    let commit_head = get_commit(&head)?;

    // Merge the trees
    read_tree_merged(&commit_base.tree, &commit_head.tree, &commit_other.tree)?;
    println!("Merged in working tree\nPlease commit");

    Ok(())
}

pub fn get_merge_base(oid1: &str, oid2: &str) -> Result<String, io::Error> {
    let mut que = VecDeque::new();
    que.push_back(String::from(oid1));

    let parents1: HashSet<String> = iter_commits_and_parents(que)?.into_iter().collect();
    que = VecDeque::new();
    que.push_back(String::from(oid2));
    for oid in iter_commits_and_parents(que)? {
        if parents1.contains(&oid) {
            return Ok(oid);
        }
    }
    Ok(String::new())
}