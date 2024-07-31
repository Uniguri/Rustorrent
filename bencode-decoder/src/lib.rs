use std::collections::HashMap;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Element {
    ByteString(Vec<u8>),
    Integer(i64),
    List(Vec<Element>),
    Dictionary(HashMap<String, Element>),
}

/// Decode slice to u64.
/// This function does not allow numbers starting with '+'.
///
/// # Arguments
///
/// * `ascii_num` - slice to decode
/// * `len` - the length of decoded ascii number (if decoding fails it means max length of correct ascii numbers)
///
/// # Example
///
/// - `decode_u64("1234", &mut len)` returns `Some(1234)` and `len` must be 4.
/// - `decode_u64("1234+1234", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("1234abcd", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("-1234", &mut len)` returns `None` and `len` must be 0.
/// - `decode_u64("+1234", &mut len)` returns `None` and `len` must be 0.
/// - `decode_u64("18446744073709551615", &mut len)` returns `Some(18446744073709551615)` and `len` must be 20. Note that `18446744073709551615` is `u64::MAX`.
/// - `decode_u64("18446744073709551616", &mut len)` returns `None` and `len` must be 20. Note that `18446744073709551616` is `u64::MAX + 1`.
fn decode_u64(ascii_num: &[u8], len: &mut usize) -> Option<u64> {
    if ascii_num.len() == 0 {
        *len = 0;
        return None;
    } else if ascii_num.len() >= 2 && ascii_num[0] == b'0' {
        *len = 1;
        return Some(0);
    }

    let mut num: u64 = 0;
    for cur in ascii_num {
        match *cur {
            b'0'..=b'9' => {
                *len += 1;

                num = num.checked_mul(10)?;
                num = num.checked_add((cur - b'0') as u64)?;
            }
            _ => {
                break;
            }
        }
    }

    if num == 0 && (*len == 0 || *len > 1) {
        return None;
    }

    return Some(num);
}

/// Decode slice to i64.
/// This function does not allow number starting with '+'.
///
/// # Arguments
///
/// * `ascii_num` - slice to decode
/// * `len` - the length of decoded ascii number (if decoding fails it means max length of correct ascii numbers)
///
/// # Example
///
/// For convenience, below examples use `ascii_num` as `&str`.
///
/// - `decode_u64("1234", &mut len)` returns `Some(1234)` and `len` must be 4.
/// - `decode_u64("1234+1234", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("1234abcd", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("-1234", &mut len)` returns `Some(-1234)` and `len` must be 5.
/// - `decode_u64("+1234", &mut len)` returns `None` and `len` must be 0. Note that this functions does not allow number starting with '+'.
/// - `decode_u64("9223372036854775807", &mut len)` returns `Some(9223372036854775807)` and `len` must be 19. Note that `9223372036854775807` is `i64::MAX`.
/// - `decode_u64("-9223372036854775808", &mut len)` returns `Some(-9223372036854775808)` and `len` must be 20. Note that `-9223372036854775808` is `i64::MIN`.
/// - `decode_u64("9223372036854775808", &mut len)` returns `None` and `len` must be 19. Note that `9223372036854775807` is `i64::MAX + 1`.
/// - `decode_u64("-9223372036854775809", &mut len)` returns `None` and `len` must be 20. Note that `-9223372036854775809` is `i64::MIN - 1`.
fn decode_i64(ascii_num: &[u8], len: &mut usize) -> Option<i64> {
    if ascii_num.len() == 0 {
        return None;
    }

    let is_positive;
    let start_offs: usize;
    if ascii_num[0] == b'-' {
        is_positive = false;
        start_offs = 1;
    } else {
        is_positive = true;
        start_offs = 0;
    }

    let opt_num = decode_u64(&ascii_num[start_offs..], len);
    *len += start_offs;
    let num;
    match opt_num {
        Some(x) => {
            num = x;
        }
        None => {
            return None;
        }
    }

    if is_positive == true {
        if num <= i64::MAX as u64 {
            return Some(num as i64);
        } else {
            return None;
        }
    } else {
        match num.cmp(&(i64::MIN as u64)) {
            std::cmp::Ordering::Less => {
                if num == 0 {
                    return None;
                }
                return Some(-(num as i64));
            }
            std::cmp::Ordering::Equal => {
                return Some(i64::MIN);
            }
            std::cmp::Ordering::Greater => {
                return None;
            }
        }
    }
}

fn decode_bytesstring(bencode: &[u8], len: &mut usize) -> Option<Element> {
    if bencode.len() == 0 {
        *len = 0;
        return None;
    }

    let mut bytes_len_len = 0;
    let bytes_len = decode_u64(&bencode[0..], &mut bytes_len_len)? as usize;
    let start_idx = bytes_len_len + 1;
    let end_idx = start_idx + bytes_len;
    if start_idx > bencode.len() || bencode[bytes_len_len] != b':' || end_idx > bencode.len() {
        return None;
    }

    let bytes = &bencode[start_idx..(end_idx)];
    *len = end_idx;
    return Some(Element::ByteString(bytes.to_vec()));
}

fn decode_integer(bencode: &[u8], len: &mut usize) -> Option<Element> {
    if bencode.len() < 3 || bencode[0] != b'i' {
        *len = 0;
        return None;
    }

    let mut int_len = 0;
    let int = decode_i64(&bencode[1..], &mut int_len)?;
    if 1 + int_len >= bencode.len() || bencode[1 + int_len] != b'e' {
        return None;
    }
    *len = int_len + 2;
    return Some(Element::Integer(int));
}

fn decode_list(bencode: &[u8], len: &mut usize) -> Option<Element> {
    if bencode.len() < 2 || bencode[0] != b'l' {
        *len = 0;
        return None;
    }

    let mut list = Vec::<Element>::new();
    let mut idx = 1;
    while idx < bencode.len() && bencode[idx] != b'e' {
        let mut list_len = 0;
        let elem_in_list = decode_all(&bencode[idx..], &mut list_len)?;
        idx += list_len;
        list.push(elem_in_list);
    }

    if bencode[idx] != b'e' {
        *len = idx;
        return None;
    }
    *len = idx + 1;
    return Some(Element::List(list));
}

fn decode_dictionary(bencode: &[u8], len: &mut usize) -> Option<Element> {
    if bencode.len() < 2 || bencode[0] != b'd' {
        *len = 0;
        return None;
    }

    let mut dict = HashMap::<String, Element>::new();
    let mut idx = 1;
    while idx < bencode.len() && bencode[idx] != b'e' {
        let mut key_len = 0;
        let dict_key;
        if let Some(Element::ByteString(key)) = decode_bytesstring(&bencode[idx..], &mut key_len) {
            dict_key = String::from_utf8(key).ok()?;
            idx += key_len;
            if idx >= bencode.len() {
                return None;
            }
        } else {
            return None;
        }
        let mut val_len = 0;
        let dict_val = decode_all(&bencode[idx..], &mut val_len)?;
        idx += val_len;
        dict.insert(dict_key, dict_val);
    }

    if bencode[idx] != b'e' {
        return None;
    }
    *len = idx + 1;
    return Some(Element::Dictionary(dict));
}

fn decode_all(bencode: &[u8], len: &mut usize) -> Option<Element> {
    if bencode.len() == 0 {
        return None;
    }

    match bencode[0] {
        b'0'..=b'9' => {
            return decode_bytesstring(bencode, len);
        }
        b'i' => {
            return decode_integer(bencode, len);
        }
        b'l' => {
            return decode_list(bencode, len);
        }
        b'd' => {
            return decode_dictionary(bencode, len);
        }
        b'e' | _ => {
            return None;
        }
    }
}

#[allow(dead_code)]
/// Decode bencoded data.
///
/// # Arguments
/// * `bencode` - bencoded data **without** spaces.
fn decode(bencode: &[u8]) -> Option<Element> {
    let mut len = 0;
    let ret = decode_all(bencode, &mut len);
    if len != bencode.len() {
        return None;
    }
    return ret;
}
#[cfg(test)]
mod tests {
    use super::*;

    mod decode_u64_test {
        use super::*;

        fn helper(input: &str, expect: Option<u64>, expected_len: usize) {
            let mut len: usize = 0;
            let result = decode_u64(input.as_bytes(), &mut len);
            assert_eq!(result, expect);
            assert_eq!(len, expected_len);
        }

        #[test]
        fn decode_u64_01() {
            let s = "1234";
            helper(s, Some(1234), s.len());
        }

        #[test]
        fn decode_u64_02() {
            let s = "0";
            helper(s, Some(0), s.len());
        }

        #[test]
        fn decode_u64_03() {
            let s = "";
            helper(s, None, s.len());
        }

        #[test]
        fn decode_u64_04() {
            let s = "-1";
            helper(s, None, 0);
        }

        #[test]
        fn decode_u64_05() {
            let ss = u64::MAX.to_string();
            let s = ss.as_str();
            helper(s, Some(u64::MAX), s.len());
        }

        #[test]
        fn decode_u64_06() {
            let ss = (u64::MAX as u128 + 1).to_string();
            let s = ss.as_str();
            helper(s, None, s.len());
        }

        #[test]
        fn decode_u64_07() {
            let s = "1234+1234";
            helper(s, Some(1234), 4);
        }

        #[test]
        fn decode_u64_08() {
            let s = "01";
            helper(s, Some(0), 1);
        }

        #[test]
        fn decode_u64_09() {
            let s = "000abcd";
            helper(s, Some(0), 1);
        }

        #[test]
        fn decode_u64_10() {
            let s = "0abcd";
            helper(s, Some(0), 1);
        }
    }

    mod decode_i64_test {
        use super::*;

        fn helper(input: &str, expect: Option<i64>, expected_len: usize) {
            let mut len: usize = 0;
            let result = decode_i64(input.as_bytes(), &mut len);
            assert_eq!(result, expect);
            assert_eq!(len, expected_len);
        }

        #[test]
        fn decode_i64_01() {
            let s = "1234";
            helper(s, Some(1234), s.len());
        }

        #[test]
        fn decode_i64_02() {
            let s = "0";
            helper(s, Some(0), s.len());
        }

        #[test]
        fn decode_i64_03() {
            let s = "";
            helper(s, None, s.len());
        }

        #[test]
        fn decode_i64_05() {
            let s = "-1234";
            helper(s, Some(-1234), s.len());
        }

        #[test]
        fn decode_i64_06() {
            let s = "1234-56";
            helper(s, Some(1234), 4);
        }

        #[test]
        fn decode_i64_07() {
            let s = "+1234";
            helper(s, None, 0); // +<integer> is not allowed.
        }

        #[test]
        fn decode_i64_08() {
            let ss = i64::MAX.to_string();
            let s = ss.as_str();
            helper(s, Some(i64::MAX), s.len());
        }

        #[test]
        fn decode_i64_09() {
            let ss = (i64::MAX as u64 + 1).to_string();
            let s = ss.as_str();
            helper(s, None, s.len());
        }

        #[test]
        fn decode_i64_10() {
            let ss = i64::MIN.to_string();
            let s = ss.as_str();
            helper(s, Some(i64::MIN), s.len());
        }

        #[test]
        fn decode_i64_11() {
            let ss = (i64::MIN as i128 - 1).to_string();
            let s = ss.as_str();
            helper(s, None, s.len());
        }

        #[test]
        fn decode_i64_12() {
            let s = "-1234+1234";
            helper(s, Some(-1234), 5);
        }

        #[test]
        fn decode_i64_13() {
            let s = "-0";
            helper(s, None, 2);
        }

        #[test]
        fn decode_i64_14() {
            let s = "-+";
            helper(s, None, 1);
        }

        #[test]
        fn decode_i64_15() {
            let s = "--+1234";
            helper(s, None, 1);
        }
    }

    mod decode_test {
        use super::*;

        fn helper(input: &str, expect: Option<Element>) {
            let result = decode(input.as_bytes());
            assert_eq!(result, expect);
        }

        #[test]
        fn decode_01() {
            helper("0:", Some(Element::ByteString(Vec::<u8>::new())));
        }

        #[test]
        fn decode_02() {
            helper(
                "5:a cde",
                Some(Element::ByteString(vec![b'a', b' ', b'c', b'd', b'e'])),
            );
        }

        #[test]
        fn decode_03() {
            helper("5:abcdef", None);
        }

        #[test]
        fn decode_04() {
            helper("10:abcdef", None);
        }

        #[test]
        fn decode_05() {
            helper("i0e", Some(Element::Integer(0)));
        }

        #[test]
        fn decode_06() {
            helper("i-0e", None);
        }

        #[test]
        fn decode_07() {
            helper("i-10e", Some(Element::Integer(-10)));
        }

        #[test]
        fn decode_08() {
            helper("i1234e", Some(Element::Integer(1234)));
        }

        #[test]
        fn decode_09() {
            helper("i0123e", None);
        }

        #[test]
        fn decode_10() {
            helper("le", Some(Element::List(Vec::<Element>::new())));
        }

        #[test]
        fn decode_11() {
            helper(
                "li1ei2ee",
                Some(Element::List(vec![
                    Element::Integer(1),
                    Element::Integer(2),
                ])),
            );
        }

        #[test]
        fn decode_12() {
            helper(
                "li1e2:ablee",
                Some(Element::List(vec![
                    Element::Integer(1),
                    Element::ByteString(vec![b'a', b'b']),
                    Element::List(Vec::<Element>::new()),
                ])),
            );
        }

        #[test]
        fn decode_13() {
            helper(
                "de",
                Some(Element::Dictionary([].iter().cloned().collect())),
            );
        }

        #[test]
        fn decode_14() {
            helper(
                "d1:a1:be",
                Some(Element::Dictionary(
                    [("a".to_string(), Element::ByteString(vec![b'b']))]
                        .iter()
                        .cloned()
                        .collect(),
                )),
            );
        }

        #[test]
        fn decode_15() {
            helper(
                "d1:a1:b1:bde1:cli1234e2:abee",
                Some(Element::Dictionary(
                    [
                        ("a".to_string(), Element::ByteString(vec![b'b'])),
                        (
                            "b".to_string(),
                            Element::Dictionary([].iter().cloned().collect()),
                        ),
                        (
                            "c".to_string(),
                            Element::List(vec![
                                Element::Integer(1234),
                                Element::ByteString(vec![b'a', b'b']),
                            ]),
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )),
            );
        }
    }
}
