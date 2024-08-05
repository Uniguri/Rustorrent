use std::collections::HashMap;

mod utils;

use crate::utils::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Element {
    ByteString(Vec<u8>),
    Integer(i64),
    List(Vec<Element>),
    Dictionary(HashMap<String, Element>),
}

#[allow(dead_code)]
impl Element {
    pub fn convert_to_ref_vec_u8(&self) -> Option<&Vec<u8>> {
        if let Element::ByteString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn convert_to_str(&self) -> Option<&str> {
        if let Element::ByteString(x) = self {
            core::str::from_utf8(x).ok()
        } else {
            None
        }
    }

    pub fn convert_to_string(&self) -> Option<String> {
        Some(self.convert_to_str()?.to_string())
    }

    pub fn convert_to_i64(&self) -> Option<i64> {
        if let Element::Integer(x) = self {
            Some(*x)
        } else {
            None
        }
    }

    pub fn convert_to_u64(&self) -> Option<u64> {
        if let Some(x) = self.convert_to_i64() {
            Some(x as u64)
        } else {
            None
        }
    }

    pub fn convert_to_string_list(&self) -> Option<Vec<String>> {
        if let Element::List(x) = self {
            x.iter().map(|y| y.convert_to_string()).collect()
        } else {
            None
        }
    }

    pub fn convert_to_ref_list(&self) -> Option<&Vec<Element>> {
        if let Element::List(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn convert_to_list(&self) -> Option<Vec<Element>> {
        match self.convert_to_ref_list() {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn convert_to_ref_dict(&self) -> Option<&HashMap<String, Element>> {
        if let Element::Dictionary(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn convert_to_dict(&self) -> Option<HashMap<String, Element>> {
        match self.convert_to_ref_dict() {
            Some(x) => Some(x.clone()),
            None => None,
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
        let dict_key = decode_bytesstring(&bencode[idx..], &mut key_len)?.convert_to_string()?;
        idx += key_len;
        if idx >= bencode.len() {
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
/// The length of decoded data must be same as the length of input.
///
/// # Arguments
/// * `bencode` - bencoded data **without** spaces.
pub fn decode_len_check(bencode: &[u8]) -> Option<Element> {
    let mut len = 0;
    let ret = decode_all(bencode, &mut len);
    if len != bencode.len() {
        return None;
    }
    return ret;
}

#[allow(dead_code)]
/// Decode bencoded data.
///
/// # Arguments
/// * `bencode` - bencoded data **without** spaces.
pub fn decode_no_len_check(bencode: &[u8]) -> Option<Element> {
    let mut len = 0;
    decode_all(bencode, &mut len)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod decode_len_check_test {
        use super::*;

        fn helper(input: &str, expect: Option<Element>) {
            let result = decode_len_check(input.as_bytes());
            assert_eq!(result, expect);
        }

        #[test]
        fn decode_len_check_01() {
            helper("0:", Some(Element::ByteString(Vec::<u8>::new())));
        }

        #[test]
        fn decode_len_check_02() {
            helper(
                "5:a cde",
                Some(Element::ByteString(vec![b'a', b' ', b'c', b'd', b'e'])),
            );
        }

        #[test]
        fn decode_len_check_03() {
            helper("5:abcdef", None);
        }

        #[test]
        fn decode_len_check_04() {
            helper("10:abcdef", None);
        }

        #[test]
        fn decode_len_check_05() {
            helper("i0e", Some(Element::Integer(0)));
        }

        #[test]
        fn decode_len_check_06() {
            helper("i-0e", None);
        }

        #[test]
        fn decode_len_check_07() {
            helper("i-10e", Some(Element::Integer(-10)));
        }

        #[test]
        fn decode_len_check_08() {
            helper("i1234e", Some(Element::Integer(1234)));
        }

        #[test]
        fn decode_len_check_09() {
            helper("i0123e", None);
        }

        #[test]
        fn decode_len_check_10() {
            helper("le", Some(Element::List(Vec::<Element>::new())));
        }

        #[test]
        fn decode_len_check_11() {
            helper(
                "li1ei2ee",
                Some(Element::List(vec![
                    Element::Integer(1),
                    Element::Integer(2),
                ])),
            );
        }

        #[test]
        fn decode_len_check_12() {
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
        fn decode_len_check_13() {
            helper(
                "de",
                Some(Element::Dictionary([].iter().cloned().collect())),
            );
        }

        #[test]
        fn decode_len_check_14() {
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
        fn decode_len_check_15() {
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
