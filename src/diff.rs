use core::error;
use std::{collections::HashMap, error::Error, io::{self, Write}, process::{Command, Stdio}};

use tempfile::NamedTempFile;

use crate::data;


type Tree = HashMap<String, String>;

pub fn compare_trees(trees: Vec<Tree>) -> HashMap<String, Vec<Option<String>>> {
    let mut entries: HashMap<String, Vec<Option<String>>> = HashMap::new();

    for (i, tree) in trees.iter().enumerate() {
        for (path, oid) in tree {
            let entry = entries.entry(path.clone()).or_insert_with(|| vec![None; trees.len()]);
            entry[i] = Some(oid.clone());
        }
    }
    
    entries
}

pub fn diff_trees(t_from: Tree, t_to: Tree) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();

    for (path, oids) in compare_trees(vec![t_from, t_to]) {
        let o_from = oids[0].clone();
        let o_to = oids[1].clone();

        if o_from != o_to {
            output.extend(diff_blobs(o_from, o_to, &path)?);
        }
    }

    Ok(output)
}

pub fn iter_changed_files(t_from: Tree, t_to: Tree) -> Vec<(String, String)> {
    let mut changed_files = Vec::new();

    for (path, oids) in compare_trees(vec![t_from, t_to]) {
        let o_from = oids[0].clone();
        let o_to = oids[1].clone();

        if o_from != o_to {
            let action = if o_from.is_none() {
                "new file".to_string()
            } else if o_to.is_none() {
                "deleted".to_string()
            } else {
                "modified".to_string()
            };
            changed_files.push((path, action));
        }
    }

    changed_files
}


pub fn diff_blobs(o_from: Option<String>, o_to: Option<String>, path: &str) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();

    // Create temporary files for the diff
    let mut f_from = NamedTempFile::new()?;
    let mut f_to = NamedTempFile::new()?;

    // Write blobs to files
    if let Some(oid) = o_from {
        let blob = data::get_object(&oid, "blob")?;
        f_from.write_all(&blob.as_bytes())?;
        f_from.flush()?;
    }

    if let Some(oid) = o_to {
        let blob = data::get_object(&oid, "blob")?;
        f_to.write_all(&blob.as_bytes())?;
        f_to.flush()?;
    }

    // Run the `diff` command using subprocess
    let diff_output = Command::new("diff")
        .arg("--unified")
        .arg("--show-c-function")
        .arg("--label")
        .arg(format!("a/{}", path))
        .arg(f_from.path())
        .arg("--label")
        .arg(format!("b/{}", path))
        .arg(f_to.path())
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute diff");

    output.extend(diff_output.stdout);

    Ok(output)
}

pub fn merge_trees(t_base: Tree, t_HEAD: Tree, t_other: Tree) -> Result<HashMap<String, Vec<u8>>, io::Error> {
    let mut tree = HashMap::new();

    for (path, oids) in compare_trees(vec![t_base, t_HEAD, t_other]) {
        let o_base = oids[0].clone();
        let o_HEAD = oids[1].clone();
        let o_other = oids[2].clone();

        let merged_blob = merge_blobs(o_base, o_HEAD, o_other)?;
        tree.insert(path, merged_blob);
    }

    Ok(tree)
}

pub fn merge_blobs(o_base: Option<String>, o_HEAD: Option<String>, o_other: Option<String>) -> Result< Vec<u8>, io::Error> {
    let mut f_base = NamedTempFile::new()?;
    let mut f_HEAD = NamedTempFile::new()?;
    let mut f_other = NamedTempFile::new()?;

    if let Some(oid) = o_base {
        let blob = data::get_object(&oid, "blob")?;
        f_base.write_all(&blob.as_bytes())?;
        f_base.flush()?;
    }

    if let Some(oid) = o_HEAD {
        let blob = data::get_object(&oid, "blob")?;
        f_HEAD.write_all(&blob.as_bytes())?;
        f_HEAD.flush()?;
    }

    if let Some(oid) = o_other {
        let blob = data::get_object(&oid, "blob")?;
        f_other.write_all(&blob.as_bytes())?;
        f_other.flush()?;
    }

    let diff_output = Command::new("diff3")
        .arg("-m")
        .arg("-L").arg("HEAD").arg(f_HEAD.path())
        .arg("-L").arg("BASE").arg(f_base.path())
        .arg("-L").arg("MERGE_HEAD").arg(f_other.path())
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute diff3");

    assert!(diff_output.status.success() || diff_output.status.code() == Some(1));

    Ok(diff_output.stdout)
}