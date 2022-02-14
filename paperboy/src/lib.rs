pub mod error;

pub mod mailer;
mod paperboy;
mod rss;
pub mod subscriptions;

/// Alias for a `Result` with the error type `paperboy::error::Error`.
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

pub use mailer::{Config, Credentials, Mailer};
pub use paperboy::Paperboy;
pub use rss::{Entry, Feed, FeedLoader};

#[cfg(test)]
pub mod test_util {
    use std::{
        fs::{create_dir_all, File},
        io::Write,
        panic,
    };

    use rand::{distributions::Alphanumeric, Rng};

    pub fn run<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        let result = panic::catch_unwind(|| test());
        assert!(result.is_ok())
    }

    pub fn create_tmp_file(content: &str) -> (String, String) {
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
}
