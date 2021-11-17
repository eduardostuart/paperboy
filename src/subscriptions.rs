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
    use std::{
        fs::{self, create_dir_all, File},
        io::Write,
        panic,
    };

    use rand::{distributions::Alphanumeric, Rng};

    use super::*;

    #[test]
    fn should_be_able_to_load_subscriptions() {
        let (path, file_path) = create_tmp_test_file(
            r#"
https://a/feed
https://b/feed"#,
        );

        let result = panic::catch_unwind(|| {
            let subs = load_from_file(&file_path);
            assert_eq!(2, subs.clone().len());
            assert_eq!(subs.clone().into_iter().nth(0).unwrap(), "https://a/feed");
            assert_eq!(subs.into_iter().nth(1).unwrap(), "https://b/feed");
        });

        fs::remove_dir_all(path).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn should_ignore_empty_lines() {
        let (path, file_path) = create_tmp_test_file(
            r#"
https://a/feed


https://b/feed

https://c/feed"#,
        );

        let result = panic::catch_unwind(|| {
            let read_line = |vec: Vec<String>, row: usize| vec.into_iter().nth(row);
            let subs = load_from_file(&file_path);
            assert_eq!(3, subs.clone().len());
            assert_eq!(read_line(subs.clone(), 0).unwrap(), "https://a/feed");
            assert_eq!(read_line(subs.clone(), 1).unwrap(), "https://b/feed");
            assert_eq!(read_line(subs, 2).unwrap(), "https://c/feed");
        });

        fs::remove_dir_all(path).unwrap();

        assert!(result.is_ok());
    }

    fn create_tmp_test_file(content: &str) -> (String, String) {
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let path = format!(".tmp{}", random);
        let file_path = format!("{}/file.txt", &path);

        create_dir_all(&path).unwrap();

        let file = File::create(&file_path).unwrap();
        write!(&file, "{}", &content).unwrap();

        (String::from(path), file_path.to_string())
    }

    #[test]
    fn should_ignore_items_starting_with_hash() {
        let (path, file_path) = create_tmp_test_file(
            r#"
https://a/feed
#https://b/feed
https://c/feed
#https://d/feed"#,
        );

        let result = panic::catch_unwind(|| {
            let subs = load_from_file(&file_path);
            assert_eq!(2, subs.clone().len());
            assert_eq!(subs.clone().into_iter().nth(0).unwrap(), "https://a/feed");
            assert_eq!(subs.into_iter().nth(1).unwrap(), "https://c/feed");
        });

        fs::remove_dir_all(path).unwrap();

        assert!(result.is_ok());
    }
}
