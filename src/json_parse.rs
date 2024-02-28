use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub package_name: String,
    pub file_name: String,
    pub version: String,
    pub description: String,
    pub hash: String,
}

impl PackageInfo {
    pub fn new(
        package_name: String,
        file_name: String,
        version: String,
        description: String,
        hash: String,
    ) -> PackageInfo {
        PackageInfo {
            package_name,
            file_name,
            version,
            description,
            hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashInfo {
    pub file_name: String,
    pub hash: String,
}
impl HashInfo {
    pub fn new(file_name: String, hash: String) -> HashInfo {
        HashInfo { file_name, hash }
    }
}

pub struct JsonStorage<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> JsonStorage<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn from_json(path: &Path) -> io::Result<T> {
        let mut file_contents = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_contents)?;
        let data: T = serde_json::from_str(&file_contents)?;
        Ok(data)
    }

    pub fn to_json(data: &T, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        to_writer_pretty(file, &data)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub file_name: String,
    pub version: String,
    pub description: String,
    pub hash: String,
    pub url: String,
}

impl RepoInfo {
    pub fn new(
        file_name: String,
        version: String,
        description: String,
        hash: String,
        url: String,
    ) -> RepoInfo {
        RepoInfo {
            file_name,
            version,
            description,
            hash,
            url,
        }
    }
}
