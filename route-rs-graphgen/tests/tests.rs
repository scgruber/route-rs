use std::path::Path;
use std::{env, process, fs};

#[test]
fn trivial_identity() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new("../examples/trivial-identity"));
    assert!(crate_dir.is_dir());

    let pipeline_module = crate_dir.join(Path::new("src/pipeline.rs"));
    assert!(pipeline_module.is_file());

    let pipeline_xml = crate_dir.join(Path::new("src/pipeline.xml"));
    assert!(pipeline_xml.is_file());

    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let exe = exe_dir.join("route-rs-graphgen");

    let mut cmd = process::Command::new(exe);
    let test_temp_dir = env::temp_dir().join("trivial_identity");
    fs::create_dir_all(&test_temp_dir);

    cmd.arg(format!("--graph {}", pipeline_xml.to_str().unwrap()));
    cmd.arg(format!("--output {}/pipeline.rs", test_temp_dir.to_str().unwrap()));
    cmd.arg("--modules packets");
    cmd.arg("--rustfmt");

    assert!(cmd.status().unwrap().success());
}