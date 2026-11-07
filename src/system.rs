pub fn show_status() {
    println!("Core status: ONLINE");
    println!("Mode: {}", crate::config::MODE);
    println!("Trading engine: NOT INITIALIZED");
    println!("AI model: NOT LOADED");
    println!("System ready.");
}
