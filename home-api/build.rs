use std::{path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(&std::env::var("OUT_DIR").unwrap());
    let components = out.components();
    let pos = components
        .clone()
        .position(|c| c.as_os_str() == "target")
        .unwrap();
    let target = components
        .take(pos + 2)
        .collect::<PathBuf>()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();

    // Copy the database file to the target directory
    std::fs::copy("home-api.db", target + "/home-api.db").unwrap();
    // Generate the tailwind CSS file
    #[cfg(target_os = "windows")]
    let mut tailwind_command = Command::new("tailwindcss");
    #[cfg(not(target_os = "windows"))]
    let mut tailwind_command = Command::new("npx");
    #[cfg(not(target_os = "windows"))]
    tailwind_command.arg("tailwindcss");
    tailwind_command
        .arg("-i")
        .arg("input.css")
        .arg("-o")
        .arg("assets/output.css")
        .output()
        .expect("Failed to execute tailwind command, is tailwindcss in PATH?");
}
