use std::fs;
use std::path::Path;

use torrent::meta_info::MetaInfo;
use torrent::tracking_info::TrackerRequests;

#[test]
fn test_tracking_info_01() {
    let file_path = Path::new("tests").join("metainfo_testcase_01.torrent");
    let file_content = fs::read(file_path).expect("Failed to read torrent file");
    let metainfo = MetaInfo::from_u8_len_check(&file_content).expect("Failed to parse MetaInfo");

    assert_eq!(
        metainfo.announce,
        "udp://tracker.openbittorrent.com:80/announce".to_string()
    );

    let calculated_info_hash = metainfo.calculate_info_hash().expect("Failed to calculate info hash");
    let expected_info_hash = "3c8996f4fa4ee430cb2b8012e667db4597d1d572".to_string();

    assert_eq!(calculated_info_hash, expected_info_hash);
}

#[test]
fn test_tracking_info_02() {
    let file_path = Path::new("tests").join("metainfo_testcase_02.torrent");
    let file_content = fs::read(file_path).expect("Failed to read torrent file");
    let metainfo = MetaInfo::from_u8_len_check(&file_content).expect("Failed to parse MetaInfo");

    assert_eq!(
        metainfo.announce,
        "http://tracker.publicbt.com/announce".to_string()
    );

    let calculated_info_hash = metainfo.calculate_info_hash().expect("Failed to calculate info hash");
    let expected_info_hash = "b71d83bbbe2afff634b3d5769f45803cc820bca7".to_string();

    assert_eq!(calculated_info_hash, expected_info_hash);
}

#[test]
fn test_tracking_info_03() {
    let file_path = Path::new("tests").join("metainfo_testcase_03.torrent");
    let file_content = fs::read(file_path).expect("Failed to read torrent file");
    let metainfo = MetaInfo::from_u8_len_check(&file_content).expect("Failed to parse MetaInfo");

    assert_eq!(
        metainfo.announce,
        "https://torrent.ubuntu.com/announce".to_string()
    );

    let calculated_info_hash = metainfo.calculate_info_hash().expect("Failed to calculate info hash");
    let expected_info_hash = "4d29c6c02c97caad937d8a9b66b0bb1b6f7cbbfe";

    assert_eq!(calculated_info_hash, expected_info_hash);
}

#[test]
fn test_tracker_request() {
    let file_path = Path::new("tests").join("metainfo_testcase_03.torrent");
    let file_content = fs::read(file_path).expect("Failed to read torrent file");
    let metainfo = MetaInfo::from_u8_len_check(&file_content).expect("Failed to parse MetaInfo");
    let tracker_request = TrackerRequests::new(
        // TrackerRequests::calculate_sha1(&metainfo).unwrap(),
        "%4d%29%c6%c0%2c%97%ca%ad%93%7d%8a%9b%66%b0%bb%1b%6f%7c%bb%fe".to_string(),
        "12341234123412341234".to_string(),
        6881,
        0,
        0,
        metainfo.calculate_left(),
        true,
        "no_peer_id".to_string(),
        "started".to_string(),
        None,
        None,
        None,
        None,
    ).expect("Failed to create TrackerRequests");

    let base_url = metainfo.announce;

    match tracker_request.send_request_to_tracker(&base_url) {
        Ok(response) => {
            println!("Tracker Response: {}", response);

            // assert!(response.contains(""));
        },
        Err(e) => {
            panic!("Failed to get response from tracker: {}", e);
        }
    }

    assert!(false);
}
