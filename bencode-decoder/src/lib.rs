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
    pub fn convert_to_ref_vec_u8(&self) -> Result<&Vec<u8>, &str> {
        if let Element::ByteString(x) = self {
            Ok(x)
        } else {
            Err("[Error] No ByteString")
        }
    }

    pub fn convert_to_str(&self) -> Result<&str, &'static str> {
        if let Element::ByteString(x) = self {
            Ok(core::str::from_utf8(x).unwrap())
        } else {
            Err("[Error] No str")
        }
    }

    pub fn convert_to_string(&self) -> Result<String, &'static str> {
        Ok(self.convert_to_str()?.to_string())
    }

    pub fn convert_to_i64(&self) -> Result<i64, &str> {
        if let Element::Integer(x) = self {
            Ok(*x)
        } else {
            Err("[Error] No i64")
        }
    }

    pub fn convert_to_u64(&self) -> Result<u64, &str> {
        if let Ok(x) = self.convert_to_i64() {
            Ok(x as u64)
        } else {
            Err("[Error] No u64")
        }
    }

    pub fn convert_to_string_list(&self) -> Result<Vec<String>, &str> {
        if let Element::List(x) = self {
            x.iter().map(|y| y.convert_to_string()).collect()
        } else {
            Err("[Error] No String list")
        }
    }

    pub fn convert_to_ref_list(&self) -> Result<&Vec<Element>, &str> {
        if let Element::List(x) = self {
            Ok(x)
        } else {
            Err("[Error] No ref list")
        }
    }

    pub fn convert_to_list(&self) -> Result<Vec<Element>, &str> {
        match self.convert_to_ref_list() {
            Ok(x) => Ok(x.clone()),
            Err(err) => Err(err),
        }
    }

    pub fn convert_to_ref_dict(&self) -> Result<&HashMap<String, Element>, &str> {
        if let Element::Dictionary(x) = self {
            Ok(x)
        } else {
            Err("[Error] No ref dictionary")
        }
    }

    pub fn convert_to_dict(&self) -> Result<HashMap<String, Element>, &str> {
        match self.convert_to_ref_dict() {
            Ok(x) => Ok(x.clone()),
            Err(err) => Err(err),
        }
    }
}

fn decode_bytesstring(bencode: &[u8], len: &mut usize) -> Result<Element, &'static str> {
    if bencode.len() == 0 {
        *len = 0;
        return Err("[Error] No data");
    }

    let mut bytes_len_len = 0;
    let bytes_len = decode_u64(&bencode[0..], &mut bytes_len_len)? as usize;
    let start_idx = bytes_len_len + 1;
    let end_idx = start_idx + bytes_len;
    if start_idx > bencode.len() || bencode[bytes_len_len] != b':' || end_idx > bencode.len() {
        return Err("[Error] No data");
    }

    let bytes = &bencode[start_idx..(end_idx)];
    *len = end_idx;
    return Ok(Element::ByteString(bytes.to_vec()));
}

fn decode_integer(bencode: &[u8], len: &mut usize) -> Result<Element, &'static str> {
    if bencode.len() < 3 || bencode[0] != b'i' {
        *len = 0;
        return Err("[Error] No data");
    }

    let mut int_len = 0;
    let int = decode_i64(&bencode[1..], &mut int_len)?;
    if 1 + int_len >= bencode.len() || bencode[1 + int_len] != b'e' {
        return Err("[Error]");
    }
    *len = int_len + 2;
    return Ok(Element::Integer(int));
}

fn decode_list(bencode: &[u8], len: &mut usize) -> Result<Element, &'static str> {
    if bencode.len() < 2 || bencode[0] != b'l' {
        *len = 0;
        return Err("[Error] No data");
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
        return Err("[Error]");
    }
    *len = idx + 1;
    return Ok(Element::List(list));
}

fn decode_dictionary(bencode: &[u8], len: &mut usize) -> Result<Element, &'static str> {
    if bencode.len() < 2 || bencode[0] != b'd' {
        *len = 0;
        return Err("[Error] No data");
    }

    let mut dict = HashMap::<String, Element>::new();
    let mut idx = 1;
    while idx < bencode.len() && bencode[idx] != b'e' {
        let mut key_len = 0;
        let dict_key = decode_bytesstring(&bencode[idx..], &mut key_len)?.convert_to_string()?;
        idx += key_len;
        if idx >= bencode.len() {
            return Err("[Error] No data");
        }

        let mut val_len = 0;
        let dict_val = decode_all(&bencode[idx..], &mut val_len)?;
        idx += val_len;
        dict.insert(dict_key, dict_val);
    }

    if bencode[idx] != b'e' {
        return Err("[Error]");
    }
    *len = idx + 1;
    return Ok(Element::Dictionary(dict));
}

fn decode_all(bencode: &[u8], len: &mut usize) -> Result<Element, &'static str> {
    if bencode.len() == 0 {
        return Err("[Error] No data");
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
            return Err("[Error]");
        }
    }
}

#[allow(dead_code)]
/// Decode bencoded data.
/// The length of decoded data must be same as the length of input.
///
/// # Arguments
/// * `bencode` - bencoded data **without** spaces.
pub fn decode_len_check(bencode: &[u8]) -> Result<Element, &str> {
    let mut len = 0;
    let ret = decode_all(bencode, &mut len);
    if len != bencode.len() {
        return Err("[Error] No data");
    }
    return ret;
}

#[allow(dead_code)]
/// Decode bencoded data.
///
/// # Arguments
/// * `bencode` - bencoded data **without** spaces.
pub fn decode_no_len_check(bencode: &[u8]) -> Result<Element, &str> {
    let mut len = 0;
    decode_all(bencode, &mut len)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod decode_len_check_test {
        use super::*;

        fn helper(input: &str, expect: Result<Element, &str>) {
            let result = decode_len_check(input.as_bytes());
            assert_eq!(result, expect);
        }

        #[test]
        fn decode_len_check_01() {
            helper("0:", Ok(Element::ByteString(Vec::<u8>::new())));
        }

        #[test]
        fn decode_len_check_02() {
            helper(
                "5:a cde",
                Ok(Element::ByteString(vec![b'a', b' ', b'c', b'd', b'e'])),
            );
        }

        #[test]
        fn decode_len_check_03() {
            helper("5:abcdef", Err("[Error] No data"));
        }

        #[test]
        fn decode_len_check_04() {
            helper("10:abcdef", Err("[Error] No data"));
        }

        #[test]
        fn decode_len_check_05() {
            helper("i0e", Ok(Element::Integer(0)));
        }

        #[test]
        fn decode_len_check_06() {
            helper("i-0e", Err("[Error] No data"));
        }

        #[test]
        fn decode_len_check_07() {
            helper("i-10e", Ok(Element::Integer(-10)));
        }

        #[test]
        fn decode_len_check_08() {
            helper("i1234e", Ok(Element::Integer(1234)));
        }

        #[test]
        fn decode_len_check_09() {
            helper("i0123e", Err("[Error] No data"));
        }

        #[test]
        fn decode_len_check_10() {
            helper("le", Ok(Element::List(Vec::<Element>::new())));
        }

        #[test]
        fn decode_len_check_11() {
            helper(
                "li1ei2ee",
                Ok(Element::List(vec![
                    Element::Integer(1),
                    Element::Integer(2),
                ])),
            );
        }

        #[test]
        fn decode_len_check_12() {
            helper(
                "li1e2:ablee",
                Ok(Element::List(vec![
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
                Ok(Element::Dictionary([].iter().cloned().collect())),
            );
        }

        #[test]
        fn decode_len_check_14() {
            helper(
                "d1:a1:be",
                Ok(Element::Dictionary(
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
                Ok(Element::Dictionary(
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
