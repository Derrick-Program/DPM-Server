#![allow(warnings)]
use crate::*;
use anyhow::Result as AnyhowResult;
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
pub fn hasher(file_path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
pub fn hash(obj: &Hash) -> AnyhowResult<()> {
    let project_path = current_dir()?.join("Repo/src").join(&obj.packagename);
    let hashfile = &project_path.join("hashes.json");
    let project_info = &project_path.join("packageInfo.json");
    let mut hashes: HashMap<String, String> =
        JsonStorage::from_json(&hashfile).unwrap_or_else(|_| HashMap::new());
    let mut counter: i32 = 0;
    if !project_path.exists() {
        return Err(anyhow::anyhow!(
            "\nPackage: {} {}",
            format!("{}", obj.packagename).yellow(),
            "Not found!".red()
        ));
    }
    for entry in WalkDir::new(&project_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path != hashfile {
            counter += 1;
            let hash = hasher(&path)?;
            let relative_path = path.strip_prefix(&project_path).unwrap_or(path);
            println!(
                "{} {} {} {}",
                counter,
                relative_path.display().to_string().yellow(),
                "===>".green(),
                format!("{}", &hash.bold().blue()),
            );
            hashes.insert(relative_path.display().to_string(), hash);
        }
    }
    JsonStorage::to_json(&hashes, &hashfile)?;
    let mut hashes: HashMap<String, String> =
        JsonStorage::from_json(&hashfile).unwrap_or_else(|_| HashMap::new());
    counter += 1;
    let hash = hasher(&hashfile)?;
    println!(
        "{} {} {} {}",
        counter,
        &hashfile.file_name().unwrap().to_str().unwrap().yellow(),
        "===>".green(),
        format!("{}", &hash.bold().blue()),
    );
    hashes.insert(
        hashfile.file_name().unwrap().to_str().unwrap().to_string(),
        hash.clone(),
    );
    JsonStorage::to_json(&hashes, &hashfile)?;
    let mut package_info: PackageInfo = JsonStorage::from_json(&project_info)?;
    package_info.hash = hash;
    JsonStorage::to_json(&package_info, &project_info)?;
    Ok(())
}

pub fn build(obj: &Build) -> Result<()> {
    let project_path = current_dir()?.join("Repo/src").join(&obj.packagename);
    if !project_path.exists() {
        return Err(anyhow::anyhow!(
            "\nPackage: {} {}",
            format!("{}", &obj.packagename).yellow(),
            "Not found!".red()
        ));
    }
    let zip_file_path = current_dir()?
        .join("Repo")
        .join(format!("{}.zip", &obj.packagename));
    zip_folder(&project_path, &zip_file_path)?;
    Ok(())
}

pub fn init(obj: &Init) -> Result<()> {
    let project_path = current_dir()
        .unwrap()
        .join("Repo/src")
        .join(obj.name.as_str());
    if !project_path.exists() {
        create_dir_all(&project_path)?;
    } else {
        return Err(anyhow::anyhow!(
            "\n{} {}",
            format!("{}", project_path.display()).yellow(),
            format!("exists!").red()
        ));
    }
    File::create(&project_path.join(obj.entry.as_str()))?;
    let file_path = project_path.join("hashes.json");
    File::create(&file_path)?;
    let hash = hasher(&file_path)?;
    let package_info = PackageInfo::new(
        obj.name.to_string(),
        obj.entry.to_string(),
        obj.ver.to_string(),
        obj.description.to_string(),
        hash,
    );
    JsonStorage::to_json(&package_info, &project_path.join("packageInfo.json"))?;
    Ok(())
}

pub fn fix(obj: &Fix) -> Result<()> {
    match &obj.command {
        FixAction::Add(obj) => fix_add(obj)?,
        FixAction::Del(obj) => fix_del(obj)?,
    }
    Ok(())
}

fn fix_add(obj: &Add) -> Result<()> {
    let repo = current_dir()?.join("RepoInfo.json");
    let mut repo_info: Repos = JsonStorage::from_json(&repo)?;
    let path = std::env::current_dir()?
        .join("Repo/src")
        .join(&obj.project_name);
    let package = current_dir()?
        .join("Repo")
        .join(format!("{}.zip", &obj.project_name));
    if !package.exists() {
        return Err(anyhow::anyhow!(
            "\nPackage: {} {}",
            format!("{}", &package.display()).yellow(),
            "Not found!".red()
        ));
    }
    let pk_info: PackageInfo = JsonStorage::from_json(&path.join("packageInfo.json"))?;

    let data: RepoInfo = RepoInfo::new(
        format!("{}.zip", pk_info.package_name),
        pk_info.version,
        pk_info.description,
        hasher(&package)?,
        format!(
            "https://github.com/Derrick-Program/DPM-Server/raw/main/Repo/{}.zip",
            &obj.project_name
        ),
        pk_info.file_name,
    );
    repo_info.insert(obj.project_name.clone().to_string(), data);
    JsonStorage::to_json(&repo_info, &repo)?;
    Ok(())
}
fn fix_del(obj: &Del) -> Result<()> {
    let repo = current_dir()?.join("RepoInfo.json");
    let mut repo_info: Repos = JsonStorage::from_json(&repo)?;
    repo_info.remove(&obj.project_name);
    JsonStorage::to_json(&repo_info, &repo)?;
    Ok(())
}

pub fn repo_init() -> Result<()> {
    let repo = current_dir()?.join("RepoInfo.json");
    let mut repo_info: Repos = JsonStorage::from_json(&repo)?;
    let ret = find_zip_files_and_names_in_repo()?;
    for (path, name) in ret {
        let name_witout_zip = name.trim_end_matches(".zip");
        let project = current_dir()?.join("Repo/src").join(&name_witout_zip);
        if !project.exists() {
            return Err(anyhow::anyhow!(
                "\nPackage: {} {}",
                format!("{}", &name_witout_zip).yellow(),
                "Not found!".red()
            ));
        }
        let pk_info: PackageInfo = JsonStorage::from_json(&project.join("packageInfo.json"))?;
        let data: RepoInfo = RepoInfo::new(
            format!("{}.zip", pk_info.package_name),
            pk_info.version,
            pk_info.description,
            hasher(&path)?,
            format!(
                "https://github.com/Derrick-Program/DPM-Server/raw/main/Repo/{}",
                name
            ),
            pk_info.file_name,
        );
        repo_info.insert(name_witout_zip.to_string(), data);
        JsonStorage::to_json(&repo_info, &repo)?;
    }

    Ok(())
}

fn find_zip_files_and_names_in_repo() -> Result<Vec<(PathBuf, String)>> {
    let repo_dir = std::env::current_dir()?.join("Repo");
    let mut zip_files = Vec::new();
    if repo_dir.is_dir() {
        for entry in read_dir(repo_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(std::ffi::OsStr::to_str) == Some("zip") {
                if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) {
                    zip_files.push((path.clone(), file_name.to_string()));
                }
            }
        }
    }

    Ok(zip_files)
}
