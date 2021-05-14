use std::{path::PathBuf, process::Command};

const INPUT_DIR: &str = "tests/input";
const OUTPUT_DIR: &str = "tests/output";

macro_rules! test_case {
    ($name:ident, $input:expr) => {
        #[test]
        fn $name() {
            run_master($input)
        }
    };
}

fn run_master(input: &str) {
    let in_file: PathBuf = [INPUT_DIR, input].iter().collect();

    let output = input.to_string() + &".out";
    let out_file: PathBuf = [OUTPUT_DIR, &output].iter().collect();

    let expected = std::fs::read_to_string(&out_file).expect("Failed to read file");

    let actual = Command::new("cargo")
        .args(&["run", in_file.as_path().to_str().unwrap()])
        .output()
        .expect("Failed to execute process")
        .stdout;
    let actual = String::from_utf8(actual).expect("Failed to convert to string");

    assert_eq!(&expected, &actual);
}

test_case!(brk, "break.lox");
test_case!(class, "class.lox");
test_case!(counter, "counter.lox");
test_case!(expr, "expr.lox");
test_case!(function, "function.lox");
test_case!(inheritance, "inheritance.lox");
test_case!(lambda, "lambda.lox");
test_case!(loops, "loops.lox");
test_case!(scopes, "scopes.lox");
test_case!(stmts, "stmts.lox");
