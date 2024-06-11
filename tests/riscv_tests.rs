use std::{path::Path, process::Command};

fn run(path: &impl AsRef<Path>) {
    let fpgrars = Command::new("cargo")
        .args(["run", "--release", "--", "--no-video"])
        .arg(path.as_ref())
        .status();

    assert!(fpgrars.is_ok(), "Failed to run FPGRARS!");
    let status = fpgrars.unwrap();

    assert_eq!(
        status.code(),
        Some(42),
        "FPGRARS returned status {status} in test <{}>!",
        path.as_ref().display()
    );
}

#[test]
fn test_riscv() {
    let fast = std::env::var("FAST");
    if matches!(fast.as_deref(), Ok("1") | Ok("true") | Ok("t")) {
        return;
    }

    let dir = Path::new("tests/riscv-tests");
    assert!(dir.is_dir(), "riscv-tests directory not found!");

    for file in dir.read_dir().unwrap() {
        assert!(file.is_ok());
        let file = file.unwrap();

        if file.file_type().unwrap().is_file() {
            let path = file.path();
            let ext = path.extension().unwrap_or_default();
            if ext == "s" {
                run(&path);
            }
        }
    }
}
