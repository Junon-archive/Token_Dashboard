use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../index.html");
    println!("cargo:rerun-if-changed=../settings.html");
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../scripts/build-frontend.mjs");

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("..")
        .status()
        .expect("failed to run npm build for Tauri frontend assets");
    assert!(status.success(), "npm build failed for Tauri frontend assets");

    tauri_build::build();
}
