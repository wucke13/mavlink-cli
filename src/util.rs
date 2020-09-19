/// Extract String from mavlink PARAM_VALUE_DATA
pub fn to_string(input_slice: &[char]) -> String {
    input_slice
        .iter()
        .filter(|c| **c != char::from(0))
        .collect()
}

/// Extract String from mavlink PARAM_VALUE_DATA
pub fn to_char_arr(input: &str) -> [char; 16] {
    let mut result = [' '; 16];
    input.chars().enumerate().for_each(|(i, e)| result[i] = e);
    result
}
