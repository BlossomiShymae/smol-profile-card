
pub fn is_str_valid_length(value: &str, min: usize, max: usize) -> bool {
    if !(value.len() >= min) {
        return false;
    }
    if !(value.len() <= max) {
        return false;
    }
    true
}

pub fn is_str_valid_pattern(value: &str, blacklist: &str) -> bool {
    for value_c in value.chars() {
        for blacklist_c in blacklist.chars() {
            if value_c.eq(&blacklist_c)  {
                return false;
            }
        }
    }
    true
}

pub fn is_str_alphanumeric(value: &str) -> bool {
    value.chars().all(char::is_alphanumeric)
}