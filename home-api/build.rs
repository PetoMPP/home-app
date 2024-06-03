use std::process::Command;

fn main() {
    #[cfg(debug_assertions)]
    const TARGET: &str = "./target/debug";
    #[cfg(not(debug_assertions))]
    const TARGET: &str = "./target/release";

    // Copy the database file to the target directory
    std::fs::copy("home-api.db", TARGET.to_string() + "/home-api.db").unwrap();
    // Generate the tailwind CSS file
    Command::new("tailwindcss")
        .arg("-i")
        .arg("input.css")
        .arg("-o")
        .arg("output.css")
        .output()
        .expect("Failed to execute tailwind command, is tailwindcss in PATH?");
}
