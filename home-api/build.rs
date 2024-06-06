use std::process::Command;

fn main() {
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
