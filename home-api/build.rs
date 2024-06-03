fn main() {
    #[cfg(debug_assertions)]
    const TO: &str = "target/debug/home-api.db";
    #[cfg(not(debug_assertions))]
    const TO: &str = "target/release/home-api.db";
    std::fs::copy("home-api.db", TO).unwrap();
}
