use std::{collections::{HashMap, VecDeque}, env, fs::{self}, io::{self, Error}};

mod data;
mod base;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("args aren't enough")
    }
    if args[1] == "init" {
        base::init().unwrap();
        let path = match env::current_dir(){
            Ok(path)=> path,
            Err(_) => return,
        };
        println!("Initialized empty ugit repository in {}",path.display());
    }else if args[1] == "hash-object" {
        hash_object(&args[2]).unwrap();
    }else if args[1] == "cat-file" {
        println!("{}",data::get_object(&args[2],"tree").unwrap())
    }else if args[1] == "write-tree" {
        println!("{}",base::write_tree(&args[2]).unwrap())
    }else if args[1] == "read-tree" {
        base::read_tree(&args[2]).unwrap()
    }else if args[1] == "commit" {
        println!("{}",base::commit(&args[2]).unwrap())
    }else if args[1] == "log" {
        let oid = if args.len() > 2 {
            &args[2] 
        } else { 
            "" 
        };
        log(oid).unwrap();
    }else if args[1] == "checkout" {
        base::checkout(&args[2]).unwrap();
    }else if args[1] == "tag" {
        let oid = if args.len() > 3 { 
            &args[3] 
        } else { 
            "@" 
        };
        tag(&args[2], oid).unwrap()
    }else if args[1] == "k" {
        k().unwrap()
    }else if args[1] == "branch" {
        let mut name:Option<&str> = None;
        let mut start_point:Option<&str> = None;
        if args.len() > 2 {
            name = Some(&args[2])
        }
        if args.len() > 3 {
            start_point = Some(&args[3])
        }
        branch(name, start_point).unwrap();
    }else if args[1] == "status" {
        status().unwrap();
    }
    
}


fn hash_object(file_path: &str) -> Result<(), io::Error> {
    let data = fs::read(file_path)?;
    let object_id = data::hash_object(&data,"blob")?;
    println!("Object ID: {}", object_id);
    Ok(())
}

fn log(text: &str) -> Result<(), io::Error>{
    let mut oid: String = match data::get_ref("HEAD", true)?.value{
        Some(v) => v,
        None => return Err(Error::new(io::ErrorKind::InvalidData, format!("refvalue doesn't contain valid value"))),
    };
    println!("{oid}");
    if !text.is_empty() && text != "HEAD" && text != "@" {
        oid = String::from(text);
    }

    println!("{oid}");
    let mut oids = VecDeque::new();
    oids.push_back(oid);
    println!("{oids:?}");
    let mut refs: HashMap<String, Vec<String>> = HashMap::new();
    for (refname, reff) in data::iter_refs("",true)?{
        refs.entry(
            match reff.value{
                Some(val) => val.clone(),
                None => continue
            })
            .or_insert_with(Vec::new)  // Equivalent to setdefault
            .push(refname.clone());
    }
    for oid in base::iter_commits_and_parents(oids)? {
        let commit = match base::get_commit(&oid){
            Ok(val) => val,
            Err(_) => break
        };
        let refs_str = if let Some(refnames) = refs.get(&oid) {
            let joined_refs = refnames.join(", ");
            format!(" ({})", joined_refs)
        } else {
            String::new()
        };
        println!("commit {}{}\n", commit.2, refs_str);
        println!("{}",commit.2);
        
    }
    Ok(())
}

fn tag(name: &str, oid: &str) -> Result<(), io::Error>{
    base::create_tag(name, &oid)?;
    Ok(())
}

fn k() -> Result<(), io::Error> {
    let mut dot = String::from("digraph commits {\n");
    let mut oids = VecDeque::new();
    
    for (refname, r) in data::iter_refs("",false)? {
        dot += &format!("\"{}\" [shape=note]\n", refname);
        if let Some(val) = r.value {
            dot += &format!("\"{}\" -> \"{}\"\n", refname, val);
            oids.push_back(val);
        }
        
    } 

    for oid in base::iter_commits_and_parents(oids)? {
        let commit = base::get_commit(&oid)?;
        dot += &format!("\"{}\" [shape=box style=filled label=\"{}\"]\n", oid, &oid[..10]);
        dot += &format!("\"{}\" -> \"{}\"\n", oid, commit.1);
    }
    dot += "}\n";

    println!("{}", dot);
    Ok(())
}

fn branch(name: Option<&str>, start_point: Option<&str>) -> Result<(), io::Error> {
    if let None = name {
        let name = match name{
            Some(val) => val,
            None => return Err(Error::new(io::ErrorKind::InvalidData, format!("no name given")))
        };
        let start_point = match start_point{
            Some(val) => val,
            None => return Err(Error::new(io::ErrorKind::InvalidData, format!("no start point given")))
        };
        base::create_branch(name, &start_point)?;
        println!("Branch {name} created at {}", &start_point[0..10]);
    }else{
        let current = base::get_branch_name()?;
        for branch in base::iter_branch_names()? {
            let prefix = if branch == current {
                "*"
            }else {
                " "
            };
            println!("{prefix} {branch}");
        }
    }
    
    Ok(())
}

fn status () -> Result<(), io::Error> {

    let head = base::get_oid("@")?;
    let branchname = base::get_branch_name()?;
    if !branchname.is_empty() {
        println!("On branch {branchname}")
    }else {
        println!("HEAD detached at {}", &head[..10])
    }
    Ok(())
}