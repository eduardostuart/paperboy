use std::fs::read_to_string;

pub fn load_from_file(file: &str) -> Vec<String> {
    read_to_string(file)
        .unwrap()
        .lines()
        .filter_map(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(String::from(s.trim()))
            }
        })
        .collect()
}
