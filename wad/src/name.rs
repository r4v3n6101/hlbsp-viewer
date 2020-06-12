use serde::{Deserialize, Deserializer};
use std::ffi::CString;

const MAX_NAME: usize = 16;

#[derive(Deserialize)]
struct CName([u8; MAX_NAME]);

impl Into<CString> for CName {
    fn into(self) -> CString {
        let name = self.0;
        let size = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        unsafe { CString::from_vec_unchecked(name[..size].to_vec()) }
    }
}

pub fn deserialize_fixed_len_cstring<'de, D>(deserializer: D) -> Result<CString, D::Error>
where
    D: Deserializer<'de>,
{
    CName::deserialize::<D>(deserializer).map(|v| v.into())
}
