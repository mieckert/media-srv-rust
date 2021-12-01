const NM_FILES_TO_COPY: [&str; 6] = [
    "jquery/dist/jquery.slim.js",
    "bootstrap/dist/css/bootstrap.css",
    "bootstrap/dist/js/bootstrap.bundle.js",
    "bootstrap-icons/bootstrap-icons.svg",
    "video.js/dist/video.js",
    "video.js/dist/video-js.css",
];

const NM_TARGET_DIR: &str = "static/assets/";

const CLIENT_SRC_DIR: &str = "client-src/";

use std::fs::copy;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let mut package_json = PathBuf::from(&CLIENT_SRC_DIR);
    package_json.push("package.json");
    println!("cargo:rerun-if-changed={}", &package_json.to_string_lossy());
/*
    let output = Command::new("npm")
        .args(&["install"])
        .current_dir(Path::new(&CLIENT_SRC_DIR))
        .output()
        .expect("Failed to execute npm install");
    if !output.status.success() {
        panic!("Error during npm install");
    }
*/
    let nm_dir = format!("{}node_modules/", CLIENT_SRC_DIR);

    for file in &NM_FILES_TO_COPY {
        let mut source = PathBuf::from(&nm_dir);
        source.push(file);

        println!("cargo:rerun-if-changed={}", &source.to_string_lossy());

        let file_name = source.file_name().expect(&format!(
            "Failed to get filename from source file {}",
            &file
        ));
        let mut target = PathBuf::from(NM_TARGET_DIR);
        target.push(file_name);
        copy(&source, &target).expect(&format!(
            "Failed to copy file from {} to {}",
            &source.to_string_lossy(),
            &target.to_string_lossy()
        ));

        println!("cargo:rerun-if-changed={}", &target.to_string_lossy());
    }
}
