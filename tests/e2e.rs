use std::fs;
use std::process::Command;

const P: &str = "./target/debug/tbl";

#[test]
fn test_empty_file() {
    let out = Command::new(P)
        .arg("/dev/null")
        .output()
        .expect("test failed");
    assert_eq!(out.stdout.as_slice(), b"");
    assert_eq!(out.stderr.as_slice(), b"");
}

#[test]
fn test_samples() {
    let tests = vec![
        (
            vec![
                "-s",
                "--sort-by",
                "0n",
                "--sort-by-output",
                "--sort-ignore-first",
                "--decoration",
                "f",
                "-c",
                "3,0,2",
                "-l",
                "r | l | c",
                "testdata/in/numbers.csv",
            ],
            "testdata/out/numbers0.txt",
        ),
        (
            vec![
                "-t",
                "Name,First Name,Address",
                "-a",
                "-c",
                "1,0,3..",
                "testdata/in/names.txt",
            ],
            "testdata/out/names0.txt",
        ),
    ];
    for (c, r) in tests {
        let out = Command::new(P).args(c).output().expect("test failed");

        assert_eq!(String::from_utf8(out.stderr).unwrap(), "");
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            fs::read_to_string(r).unwrap()
        );
    }
}
