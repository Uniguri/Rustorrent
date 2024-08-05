pub fn is_not_whitespace(c: u8) -> bool {
    match c {
        b' ' | b'\r' | b'\n' | b'\t' => false,
        _ => true,
    }
}

pub fn erase_whitespaces(input: &[u8]) -> Vec<u8> {
    input
        .iter()
        .filter(|x| is_not_whitespace(**x))
        .copied()
        .collect()
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
/// For convenience, below examples use `ascii_num` as `&str`.
///
/// - `decode_u64("1234", &mut len)` returns `Some(1234)` and `len` must be 4.
/// - `decode_u64("1234+1234", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("1234abcd", &mut len)` returns `Some(1234)` and len must be 4.
/// - `decode_u64("-1234", &mut len)` returns `None` and `len` must be 0.
/// - `decode_u64("+1234", &mut len)` returns `None` and `len` must be 0.
/// - `decode_u64("18446744073709551615", &mut len)` returns `Some(18446744073709551615)` and `len` must be 20. Note that `18446744073709551615` is `u64::MAX`.
/// - `decode_u64("18446744073709551616", &mut len)` returns `None` and `len` must be 20. Note that `18446744073709551616` is `u64::MAX + 1`.
pub fn decode_u64(ascii_num: &[u8], len: &mut usize) -> Option<u64> {
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
pub fn decode_i64(ascii_num: &[u8], len: &mut usize) -> Option<i64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    mod erase_whitespaces_test {
        use super::*;

        fn helper(input: &str, expect: &str) {
            let result = erase_whitespaces(input.as_bytes());
            assert_eq!(result, expect.as_bytes());
        }

        #[test]
        fn erase_whitespaces_01() {
            helper("a b c   t", "abct");
        }

        #[test]
        fn erase_whitespaces_02() {
            helper("a \r b \n c", "abc");
        }
    }
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
}
