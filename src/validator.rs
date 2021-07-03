use std::path::PathBuf;

/// 是否为有效的路径
pub fn is_valid_path(path: String) -> Result<(), String> {
    if !PathBuf::from(path).exists() {
        return Err("The path does not exist, please make sure the entered directory exists".to_string());
    };
    Ok(())
}
