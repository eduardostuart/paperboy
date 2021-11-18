use std::fs::read_to_string;

pub fn load_from_file(file: &str) -> Vec<String> {
    read_to_string(file)
        .unwrap()
        .lines()
        .filter_map(|s| {
            if s.trim().is_empty() || s.starts_with('#') {
                None
            } else {
                Some(String::from(s.trim()))
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::test_util;
    use std::fs;

    use super::*;

    #[test]
    fn should_be_able_to_load_subscriptions() {
        let (path, file_path) = test_util::create_tmp_file(
            r#"
https://a/feed
https://b/feed"#,
        );

        test_util::run(|| {
            let subs = load_from_file(&file_path);
            assert_eq!(2, subs.len());
            assert_eq!(
                vec!["https://a/feed".to_string(), "https://b/feed".to_string(),],
                subs
            );
        });

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn should_ignore_empty_lines() {
        let (path, file_path) = test_util::create_tmp_file(
            r#"
https://a/feed


https://b/feed

https://c/feed"#,
        );

        test_util::run(|| {
            let subs = load_from_file(&file_path);
            assert_eq!(3, subs.len());
            assert_eq!(
                vec![
                    "https://a/feed".to_string(),
                    "https://b/feed".to_string(),
                    "https://c/feed".to_string()
                ],
                subs
            );
        });

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn should_ignore_items_starting_with_hash() {
        let (path, file_path) = test_util::create_tmp_file(
            r#"
https://a/feed
#https://b/feed
https://c/feed
#https://d/feed"#,
        );

        test_util::run(|| {
            let subs = load_from_file(&file_path);
            assert_eq!(2, subs.len());
            assert_eq!(
                vec!["https://a/feed".to_string(), "https://c/feed".to_string()],
                subs
            );
        });

        fs::remove_dir_all(path).unwrap();
    }
}
