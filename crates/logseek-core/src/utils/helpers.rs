//! Utility functions.

use std::path::PathBuf;

/// Get the application data directory (portable - next to executable)
pub fn app_data_dir() -> PathBuf {
    // Try to get executable path first (portable mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.join("data");
            // Create if doesn't exist
            let _ = std::fs::create_dir_all(&data_dir);
            return data_dir;
        }
    }
    // Fallback to current directory
    let data_dir = PathBuf::from(".").join("data");
    let _ = std::fs::create_dir_all(&data_dir);
    data_dir
}

pub fn cache_dir() -> PathBuf { 
    let dir = app_data_dir().join("cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn sessions_dir() -> PathBuf { 
    let dir = app_data_dir().join("sessions");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn workspaces_dir() -> PathBuf { 
    let dir = app_data_dir().join("workspaces");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Detect encoding of byte data and convert to UTF-8 string.
/// Supports UTF-8, UTF-8 with BOM, GBK, GB2312, GB18030, Shift_JIS, EUC-KR, etc.
pub fn bytes_to_string(data: &[u8]) -> String {
    // Empty data
    if data.is_empty() {
        return String::new();
    }
    
    // Check for UTF-8 BOM
    if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
        return String::from_utf8_lossy(&data[3..]).to_string();
    }
    
    // Check for UTF-16 BOM
    if data.len() >= 2 {
        if data[0] == 0xFF && data[1] == 0xFE {
            return encoding_rs::UTF_16LE.decode(data).0.to_string();
        }
        if data[0] == 0xFE && data[1] == 0xFF {
            return encoding_rs::UTF_16BE.decode(data).0.to_string();
        }
    }
    
    // Try UTF-8 first (most common)
    if let Ok(s) = std::str::from_utf8(data) {
        return s.to_string();
    }
    
    // Not valid UTF-8, try GBK (common in Chinese Windows)
    // GBK is a superset of GB2312 and compatible with GB18030
    let (decoded, _, had_errors) = encoding_rs::GBK.decode(data);
    if !had_errors {
        return decoded.to_string();
    }
    
    // Try Shift_JIS (Japanese)
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(data);
    if !had_errors {
        return decoded.to_string();
    }
    
    // Try EUC-KR (Korean)
    let (decoded, _, had_errors) = encoding_rs::EUC_KR.decode(data);
    if !had_errors {
        return decoded.to_string();
    }
    
    // Fallback: use GBK anyway (best effort for Chinese)
    let (decoded, _, _) = encoding_rs::GBK.decode(data);
    decoded.to_string()
}

/// Decode a single line from bytes, handling various encodings.
pub fn decode_line(bytes: &[u8]) -> String {
    bytes_to_string(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir() { 
        let dir = app_data_dir();
        assert!(dir.exists() || dir.to_string_lossy().contains("data")); 
    }

    #[test]
    fn test_cache_dir_subdir() { 
        let cache = cache_dir();
        let app = app_data_dir();
        assert!(cache.starts_with(&app));
    }

    #[test]
    fn test_workspaces_dir() {
        let ws = workspaces_dir();
        assert!(ws.to_string_lossy().contains("workspaces"));
    }

    #[test]
    fn test_utf8_detection() {
        let data = "Hello World".as_bytes();
        assert_eq!(bytes_to_string(data), "Hello World");
    }

    #[test]
    fn test_gbk_detection() {
        // "你好世界" in GBK encoding
        let data = [0xC4, 0xE3, 0xBA, 0xC3, 0xCA, 0xC0, 0xBD, 0xE7];
        let result = bytes_to_string(&data);
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn test_utf8_bom() {
        let data = [0xEF, 0xBB, 0xBF, 0x48, 0x65, 0x6C, 0x6C, 0x6F]; // BOM + "Hello"
        assert_eq!(bytes_to_string(&data), "Hello");
    }
}
