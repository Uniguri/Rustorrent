use std::fs;
use std::path::Path;

use torrent::meta_info::MetaInfo;

#[test]
fn test_meta_info_01() {
    let file_path = Path::new("tests").join("metainfo_testcase_01.torrent");
    let file_content = fs::read(file_path).unwrap();
    let metainfo = MetaInfo::from_u8_len_check(&file_content).unwrap();
    assert_eq!(
        metainfo.announce,
        "udp://tracker.openbittorrent.com:80/announce".to_string()
    );
}

#[test]
fn test_meta_info_02() {
    let file_path = Path::new("tests").join("metainfo_testcase_02.torrent");
    let file_content = fs::read(file_path).unwrap();
    let metainfo = MetaInfo::from_u8_len_check(&file_content).unwrap();
    assert_eq!(
        metainfo.announce,
        "http://tracker.publicbt.com/announce".to_string()
    );
}
