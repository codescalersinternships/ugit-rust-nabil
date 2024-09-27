use std::{collections::HashSet, env, fs::{self}, io::{self}};

mod data;
mod base;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("args aren't enough")
    }
    if args[1] == "init" {
        data::init().unwrap();
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
    }
    
}


fn hash_object(file_path: &str) -> Result<(), io::Error> {
    let data = fs::read(file_path)?;
    let object_id = data::hash_object(&data,"blob")?;
    println!("Object ID: {}", object_id);
    Ok(())
}

fn log(text: &str) -> Result<(), io::Error>{
    let mut oid: String = data::get_ref("HEAD")?;
    if !text.is_empty() && ( text != "HEAD" || text != "@") {
        oid = String::from(text);
    }
    while !oid.is_empty() {
        let commit = match base::get_commit(&oid){
            Ok(val) => val,
            Err(_) => break
        };
        println!("commit {}",oid);
        println!("{}",commit[0].2);
        oid = commit[0].1.clone();
    }
    Ok(())
}

fn tag(name: &str, oid: &str) -> Result<(), io::Error>{
    base::create_tag(name, &oid)?;
    Ok(())
}

fn k() -> Result<(), io::Error> {
    let mut dot = String::from("digraph commits {\n");
    let mut oids = HashSet::new();
    
    for (refname, r) in data::iter_refs()? {
        dot += &format!("\"{}\" [shape=note]\n", refname);
        dot += &format!("\"{}\" -> \"{}\"\n", refname, r);
        oids.insert(r);
    } 

    for oid in base::iter_commits_and_parents(oids)? {
        let commit = base::get_commit(&oid)?;
        dot += &format!("\"{}\" [shape=box style=filled label=\"{}\"]\n", oid, &oid[..10]);
        let parent = commit[0].1.clone();
        dot += &format!("\"{}\" -> \"{}\"\n", oid, parent);
    }
    dot += "}\n";

    println!("{}", dot);
    Ok(())
}