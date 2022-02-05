#[macro_use]
extern crate quote;

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

mod binder;
mod parser;

pub fn main() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Update and init submodule
    if !Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .current_dir(&src_dir)
        .status()
        .expect("Failed to update submodules")
        .success()
    {
        panic!("Git command finished with non zero code");
    }

    // Generate LogMessages
    if !Command::new("python")
        .arg("Tools/autotest/logger_metadata/parse.py")
        .arg("--vehicle")
        .arg("Plane")
        .current_dir(&src_dir.join("build/ardupilot"))
        .status()
        .expect("Failed to call parser")
        .success()
    {
        panic!("Parser finished with non zero code");
    }

    let file_path = src_dir.join("build/ardupilot/LogMessages.xml");

    println!("cargo:rerun-if-changed={}", file_path.to_string_lossy());

    let mut in_file = File::open(&file_path).unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let definition_rs = PathBuf::from("messages.rs");
    let dest_path = Path::new(&out_dir).join(&definition_rs);
    let mut out_file = File::create(&dest_path).unwrap();

    parser::generate(&mut in_file, &mut out_file);

    let dest_path = Path::new(&out_dir).join("mod.rs");
    let mut outf = File::create(&dest_path).unwrap();

    // generate code
    binder::generate(["messages".into()].to_vec(), &mut outf);

    // format code
    if !Command::new("rustfmt")
        .arg(dest_path.as_os_str())
        .arg(definition_rs.as_os_str())
        .current_dir(&out_dir)
        .status()
        .expect("Failed to call parser")
        .success()
    {
        panic!("Rust format failed!");
    }
}
