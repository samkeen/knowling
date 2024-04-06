use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn set_panic_hook(log_dir: &Path) {
    let log_file_path = log_dir.join("panic.log");
    std::panic::set_hook(Box::new(move |info| {
        if let Err(e) = write_to_file(&format!("Panicked: {:?}", info), log_file_path.clone()) {
            eprintln!("Failed to write to file: {}", e);
        }
    }));
}

pub fn get_user_app_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Unable to get the user's home directory");
    let knowling_dir = home_dir.join(".knowling");
    // Create .knowling directory if it does not exist
    if !knowling_dir.exists() {
        fs::create_dir_all(&knowling_dir).expect("Failed to create .knowling directory");
    }
    knowling_dir
}

fn write_to_file(data: &str, file_path: PathBuf) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    writeln!(file, "{}", data)
}
