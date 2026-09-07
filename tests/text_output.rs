use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn create() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "myloasm-kmc-text-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn writes_stranded_counts_to_text_and_removes_database() {
    let directory = TestDirectory::create();
    let input = directory.path().join("reads.fa");
    let output = directory.path().join("counts.tsv");
    fs::write(&input, ">forward\nAAC\n>reverse\nGTT\n").unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_myloasm-kmc-v1"))
        .args([
            "--text",
            "--output",
            output.to_str().unwrap(),
            "--tmp-dir",
            directory.path().to_str().unwrap(),
            "--kmer-size",
            "3",
            "--threads",
            "1",
            "--ram-gb",
            "2",
            "--min-count",
            "1",
            "--min-count-per-strand",
            "1",
            "--min-mid-quality",
            "0",
            input.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "command failed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read_to_string(output).unwrap(), "AAC\t1\t1\n");

    let leftovers: Vec<_> = fs::read_dir(directory.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".myloasm-kmc")
        })
        .collect();
    assert!(leftovers.is_empty(), "temporary database was not removed");
}
