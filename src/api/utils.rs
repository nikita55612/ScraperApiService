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