use core::str;
use std::collections::HashMap;

use bencode_decoder::*;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct CommonFileInfo {
    pub piece_length: usize,
    pub pieces: Vec<Vec<u8>>,
    pub is_private: bool,
}

#[allow(dead_code)]
impl CommonFileInfo {
    const PIECE_HASH_SIZE: usize = 20;

    pub fn new(piece_length: usize, pieces: &Vec<u8>, is_private: bool) -> Result<Self, &str> {
        if pieces.len() % 20 != 0 {
            return Err("No data");
        } else {
            Ok(CommonFileInfo {
                piece_length,
                pieces: pieces
                    .chunks(Self::PIECE_HASH_SIZE)
                    .map(|chk| chk.to_vec())
                    .collect(),
                is_private,
            })
        }
    }

    pub fn from_element(info_element: &Element) -> Result<Self, &str> {
        if let Element::Dictionary(dict) = info_element {
            CommonFileInfo::from_dict(dict)
        } else {
            Err("No data")
        }
    }

    pub fn from_dict(info_dict: &HashMap<String, Element>) -> Result<Self, &str> {
        let piece_length = info_dict
            .get("piece length")
            .ok_or("Missing 'piece length'")?
            .convert_to_u64()? as usize;

        let pieces = info_dict
            .get("pieces")
            .ok_or("Missing 'pieces'")?
            .convert_to_ref_vec_u8()?;

        let is_private = match info_dict.get("private") {
            Some(x) => {
                if let Ok(y) = x.convert_to_i64() {
                    y == 1
                } else {
                    false
                }
            }
            None => false,
        };

        Ok(CommonFileInfo::new(piece_length, pieces, is_private)?)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct SingleFileInfo {
    pub common_file_info: CommonFileInfo,
    pub name: String,
    pub length: usize,
    pub md5sum: Option<String>,
}

// Default trait for SingleFileInfo structure
// impl Default for SingleFileInfo {
//     fn default() -> Self {
//         SingleFileInfo {
//             common_file_info: Default::default(),
//             name: Default::default(),
//             length: Default::default(),
//             md5sum: Err("No MD5 sum"),
//         }
//     }
// }

#[allow(dead_code)]
impl SingleFileInfo {
    pub fn new_with_common_info(
        common_file_info: CommonFileInfo,
        info_dict: &HashMap<String, Element>,
    ) -> Result<Self, &str> {
        let name = info_dict.get("name").ok_or("err")?.convert_to_str()?;
        let length = info_dict.get("length").ok_or("err")?.convert_to_u64()? as usize;
        let md5sum = match info_dict.get("md5sum") {
            Some(x) => Some(x.convert_to_str()?),
            None => None,
        };

        Ok(SingleFileInfo {
            common_file_info,
            name: name.to_string(),
            length,
            md5sum: match md5sum {
                Some(x) => Some(x.to_string()),
                None => None,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct MultipleFileInfoFile {
    pub length: usize,
    pub path: Vec<String>,
    pub md5sum: Option<String>,
}

// Default trait for MultipleFileInfoFile structure
// impl Default for MultipleFileInfoFile {
//     fn default() -> Self {
//         MultipleFileInfoFile {
//             length: Default::default(),
//             path: Default::default(),
//             md5sum: Err("No MD5 sum"),
//         }
//     }
// }

#[allow(dead_code)]
impl MultipleFileInfoFile {
    pub fn new(length: usize, path: Vec<String>, md5sum: Option<&str>) -> Self {
        MultipleFileInfoFile {
            length,
            path,
            md5sum: match md5sum {
                Some(x) => Some(x.to_string()),
                None => None,
            },
        }
    }

    pub fn from_element(info_element: &Element) -> Result<Self, &str> {
        MultipleFileInfoFile::from_dict(info_element.convert_to_ref_dict()?)
    }

    pub fn from_dict(info_dict: &HashMap<String, Element>) -> Result<Self, &str> {
        let length = info_dict.get("length").ok_or("err")?.convert_to_u64()? as usize;
        let path = info_dict
            .get("path")
            .ok_or("err")?
            .convert_to_string_list()?;
        let md5sum = match info_dict.get("md5sum") {
            Some(x) => Some(x.convert_to_str()?),
            None => None,
        };
        Ok(MultipleFileInfoFile::new(length, path, md5sum))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub struct MultipleFileInfo {
    pub common_file_info: CommonFileInfo,
    pub name: String,
    pub files: Vec<MultipleFileInfoFile>,
}

#[allow(dead_code)]
impl MultipleFileInfo {
    pub fn new_with_common_info(
        common_file_info: CommonFileInfo,
        name: &str,
        files_element: &Vec<Element>,
    ) -> Result<Self, &'static str> {
        let mut info = MultipleFileInfo {
            common_file_info,
            name: name.to_string(),
            files: Vec::<MultipleFileInfoFile>::with_capacity(files_element.len()),
        };

        for file in files_element {
            let file_dict = file.convert_to_dict()?;

            let length = file_dict.get("length").ok_or("err")?.convert_to_u64()? as usize;
            let path = file_dict
                .get("path")
                .ok_or("err")?
                .convert_to_string_list()?;
            let md5sum = match file_dict.get("md5sum") {
                Some(x) => Some(x.convert_to_str()?),
                None => None,
            };

            info.files
                .push(MultipleFileInfoFile::new(length, path, md5sum));
        }

        Ok(info)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum FileInfo {
    SingleFile(SingleFileInfo),
    MultipleFile(MultipleFileInfo),
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct MetaInfo {
    pub info: FileInfo,
    pub announce: String,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub creation_date: Option<u64>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
}

#[allow(dead_code)]
impl MetaInfo {
    pub fn new(info: FileInfo, announce: &str) -> Self {
        MetaInfo {
            info,
            announce: announce.to_string(),
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
            encoding: None,
        }
    }
}

#[allow(dead_code)]
impl MetaInfo {
    pub fn from_element(element: &Element) -> Result<MetaInfo, &'static str> {
        let hashmap;
        if let Element::Dictionary(x) = element {
            hashmap = x;
        } else {
            return Err("Element is not a dictionary");
        }

        let announce = hashmap
            .get("announce")
            .ok_or("Missing 'announce' key")?
            .convert_to_str()?;
        let info_dict = hashmap
            .get("info")
            .ok_or("Missing 'info' key")?
            .convert_to_dict()?;
        let name = info_dict
            .get("name")
            .ok_or("Missing 'name' key in 'info' dictionary")?
            .convert_to_str()?;
        let common_file_info = CommonFileInfo::from_dict(&info_dict).unwrap();
        let info = match info_dict.get("files") {
            Some(files) => {
                let files = files.convert_to_ref_list()?;
                let info = MultipleFileInfo::new_with_common_info(common_file_info, name, files)?;
                FileInfo::MultipleFile(info)
            }
            None => FileInfo::SingleFile(
                SingleFileInfo::new_with_common_info(common_file_info, &info_dict).unwrap(),
            ),
        };

        let mut ret = MetaInfo::new(info, announce);
        for key in hashmap.keys() {
            match key.as_str() {
                "announce-list" => {
                    ret.announce_list = match hashmap.get(key) {
                        Some(x) => x
                            .convert_to_ref_list()?
                            .iter()
                            .map(|ve| ve.convert_to_string_list().ok())
                            .collect(),
                        None => None,
                    };
                }
                "creation date" => {
                    ret.creation_date = match hashmap.get(key) {
                        Some(x) => x.convert_to_u64().ok(),
                        None => None,
                    };
                }
                "comment" => {
                    ret.comment = match hashmap.get(key) {
                        Some(x) => x.convert_to_string().ok(),
                        None => None,
                    };
                }
                "created by" => {
                    ret.created_by = match hashmap.get(key) {
                        Some(x) => x.convert_to_string().ok(),
                        None => None,
                    };
                }
                "encoding" => {
                    ret.encoding = match hashmap.get(key) {
                        Some(x) => x.convert_to_string().ok(),
                        None => None,
                    };
                }
                _ => (),
            }
        }

        return Ok(ret);
    }

    pub fn from_u8_len_check(bencode: &[u8]) -> Result<MetaInfo, &'static str> {
        let element =
            decode_len_check(bencode).map_err(|_| "Failed to decode bencode with length check")?;
        MetaInfo::from_element(&element)
    }

    #[allow(dead_code)]
    pub fn from_u8_no_len_check(bencode: &[u8]) -> Result<MetaInfo, &'static str> {
        let element = decode_no_len_check(bencode)
            .map_err(|_| "Failed to decode bencode with length check")?;
        MetaInfo::from_element(&element)
    }
}
