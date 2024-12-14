#![allow(warnings)]
use std::{ffi::OsString, path::Path};
use chrono::Local as LocalTime;
use sha1::{Sha1, Digest};


pub fn timestamp_now() -> u64 {
    LocalTime::now().timestamp() as u64
}

pub fn gen_uuid() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn gen_token_id() -> String {
    format!("ss.{}", gen_uuid())
}

pub fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let res = hasher.finalize();
    hex::encode(res)
}

pub fn read_file<T: AsRef<Path>>(path: T) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn list_dir<T: AsRef<Path>>(dir: T) -> std::io::Result<Vec<OsString>> {
    Ok(
        std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|v| v.file_name())
            .collect::<Vec<_>>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_dir() {
        println!("{:#?}", list_dir("assets"));
        assert_eq!(true, true);
    }
}
