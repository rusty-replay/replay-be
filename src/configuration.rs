pub fn get_configuration() -> anyhow::Result<()> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    println!("Current directory: {:?}", base_path);
    Ok(())
}