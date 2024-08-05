use core::str;
use std::collections::HashMap;

use bencode_decoder::*;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub(crate) struct CommonFileInfo {
    piece_length: usize,
    pieces: Vec<Vec<u8>>,
    is_private: bool,
}

#[allow(dead_code)]
impl CommonFileInfo {
    const PIECE_HASH_SIZE: usize = 20;

    pub fn new(piece_length: usize, pieces: &Vec<u8>, is_private: bool) -> Option<Self> {
        if pieces.len() % 20 != 0 {
            return None;
        } else {
            Some(CommonFileInfo {
                piece_length,
                pieces: pieces
                    .chunks(Self::PIECE_HASH_SIZE)
                    .map(|chk| chk.to_vec())
                    .collect(),
                is_private,
            })
        }
    }

    pub fn from_element(info_element: &Element) -> Option<Self> {
        if let Element::Dictionary(dict) = info_element {
            CommonFileInfo::from_dict(dict)
        } else {
            None
        }
    }

    pub fn from_dict(info_dict: &HashMap<String, Element>) -> Option<Self> {
        let piece_length = info_dict.get("piece length")?.convert_to_u64()? as usize;
        let pieces = info_dict.get("pieces")?.convert_to_ref_vec_u8()?;
        let is_private = match info_dict.get("private") {
            Some(x) => {
                if let Some(y) = x.convert_to_i64() {
                    y == 1
                } else {
                    false
                }
            }
            None => false,
        };
        Some(CommonFileInfo::new(piece_length, pieces, is_private)?)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub(crate) struct SingleFileInfo {
    common_file_info: CommonFileInfo,
    name: String,
    length: usize,
    md5sum: Option<String>,
}

#[allow(dead_code)]
impl SingleFileInfo {
    pub fn new_with_common_info(
        common_file_info: CommonFileInfo,
        info_dict: &HashMap<String, Element>,
    ) -> Option<Self> {
        let name = info_dict.get("name")?.convert_to_str()?;
        let length = info_dict.get("length")?.convert_to_u64()? as usize;
        let md5sum = match info_dict.get("md5sum") {
            Some(x) => x.convert_to_str(),
            None => None,
        };

        Some(SingleFileInfo {
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
pub(crate) struct MultipleFileInfoFile {
    length: usize,
    path: Vec<String>,
    md5sum: Option<String>,
}

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

    pub fn from_element(info_element: &Element) -> Option<Self> {
        MultipleFileInfoFile::from_dict(info_element.convert_to_ref_dict()?)
    }

    pub fn from_dict(info_dict: &HashMap<String, Element>) -> Option<Self> {
        let length = info_dict.get("length")?.convert_to_u64()? as usize;
        let path = info_dict.get("path")?.convert_to_string_list()?;
        let md5sum = match info_dict.get("md5sum") {
            Some(x) => x.convert_to_str(),
            None => None,
        };
        Some(MultipleFileInfoFile::new(length, path, md5sum))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Default)]
pub(crate) struct MultipleFileInfo {
    common_file_info: CommonFileInfo,
    name: String,
    files: Vec<MultipleFileInfoFile>,
}

#[allow(dead_code)]
impl MultipleFileInfo {
    pub fn new_with_common_info(
        common_file_info: CommonFileInfo,
        name: &str,
        files_element: &Vec<Element>,
    ) -> Option<Self> {
        let mut info = MultipleFileInfo {
            common_file_info,
            name: name.to_string(),
            files: Vec::<MultipleFileInfoFile>::with_capacity(files_element.len()),
        };

        for file in files_element {
            let file_dict = file.convert_to_dict()?;

            let length = file_dict.get("length")?.convert_to_u64()? as usize;
            let path = file_dict.get("path")?.convert_to_string_list()?;
            let md5sum = match file_dict.get("md5sum") {
                Some(x) => x.convert_to_str(),
                None => None,
            };

            info.files
                .push(MultipleFileInfoFile::new(length, path, md5sum));
        }

        Some(info)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub(crate) enum FileInfo {
    SingleFile(SingleFileInfo),
    MultipleFile(MultipleFileInfo),
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct MetaInfo {
    info: FileInfo,
    announce: String,
    announce_list: Option<Vec<Vec<String>>>,
    creation_date: Option<u64>,
    comment: Option<String>,
    created_by: Option<String>,
    encoding: Option<String>,
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
    pub fn from_element(element: &Element) -> Option<MetaInfo> {
        let hashmap;
        if let Element::Dictionary(x) = element {
            hashmap = x;
        } else {
            return None;
        }

        let announce = hashmap.get("announce")?.convert_to_str()?;

        let info_dict = hashmap.get("info")?.convert_to_dict()?;
        let common_file_info = CommonFileInfo::from_dict(&info_dict)?;
        let name = info_dict.get("name")?.convert_to_str()?;
        let info = match info_dict.get("files") {
            Some(files) => {
                let files = files.convert_to_ref_list()?;
                let info = MultipleFileInfo::new_with_common_info(common_file_info, name, files)?;
                FileInfo::MultipleFile(info)
            }
            None => FileInfo::SingleFile(SingleFileInfo::new_with_common_info(
                common_file_info,
                &info_dict,
            )?),
        };

        let mut ret = MetaInfo::new(info, announce);
        for key in hashmap.keys() {
            match key.as_str() {
                "announce-list" => {
                    ret.announce_list = hashmap
                        .get(key)? // this must be Vec<Vec<String>>
                        .convert_to_ref_list()?
                        .iter()
                        .map(|ve| ve.convert_to_string_list())
                        .collect();
                }
                "creation date" => {
                    ret.creation_date = hashmap.get(key)?.convert_to_u64();
                }
                "comment" => {
                    ret.comment = hashmap.get(key)?.convert_to_string();
                }
                "created by" => {
                    ret.created_by = hashmap.get(key)?.convert_to_string();
                }
                "encoding" => {
                    ret.encoding = hashmap.get(key)?.convert_to_string();
                }
                _ => (),
            }
        }

        return Some(ret);
    }

    pub fn from_u8_len_check(bencode: &[u8]) -> Option<MetaInfo> {
        let element = decode_len_check(bencode)?;
        return MetaInfo::from_element(&element);
    }

    #[allow(dead_code)]
    pub fn from_u8_no_len_check(bencode: &[u8]) -> Option<MetaInfo> {
        let element = decode_no_len_check(bencode)?;
        return MetaInfo::from_element(&element);
    }
}
