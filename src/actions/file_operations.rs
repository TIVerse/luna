//! File operations
//!
//! Create, move, delete files and directories.

use crate::error::{LunaError, Result};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

/// Create a new directory
/// 
/// Creates the directory and all missing parent directories (like `mkdir -p`).
/// Idempotent - succeeds if directory already exists.
/// 
/// # Arguments
/// * `path` - Path to the directory to create
/// 
/// # Errors
/// Returns `LunaError::Io` if creation fails due to permissions or other I/O errors.
pub async fn create_directory(path: &Path) -> Result<()> {
    debug!("Creating directory: {:?}", path);
    
    // Check if already exists
    if path.exists() {
        if path.is_dir() {
            debug!("Directory already exists: {:?}", path);
            return Ok(());
        } else {
            return Err(LunaError::SystemOperation(format!(
                "Path exists but is not a directory: {:?}",
                path
            )));
        }
    }
    
    // Create directory and all parent directories
    fs::create_dir_all(path).await.map_err(|e| {
        LunaError::SystemOperation(format!(
            "Failed to create directory {:?}: {}",
            path, e
        ))
    })?;
    
    info!("✅ Created directory: {:?}", path);
    Ok(())
}

/// Move a file or directory
/// 
/// Moves/renames files and directories. Handles cross-device moves by
/// falling back to copy+delete if needed.
/// 
/// # Arguments
/// * `from` - Source path
/// * `to` - Destination path
/// 
/// # Errors
/// Returns `LunaError` if source doesn't exist, destination already exists,
/// or the operation fails due to permissions.
pub async fn move_file(from: &Path, to: &Path) -> Result<()> {
    debug!("Moving {:?} to {:?}", from, to);
    
    // Validate source exists
    if !from.exists() {
        return Err(LunaError::FileNotFound(format!(
            "Source path does not exist: {:?}",
            from
        )));
    }
    
    // Check if destination already exists
    if to.exists() {
        return Err(LunaError::SystemOperation(format!(
            "Destination already exists: {:?}",
            to
        )));
    }
    
    // Ensure destination parent directory exists
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await.map_err(|e| {
                LunaError::SystemOperation(format!(
                    "Failed to create parent directory {:?}: {}",
                    parent, e
                ))
            })?;
        }
    }
    
    // Try direct rename first (fast, atomic)
    match fs::rename(from, to).await {
        Ok(_) => {
            info!("✅ Moved {:?} to {:?}", from, to);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            // Cross-device move: copy then delete
            warn!("Cross-device move detected, using copy+delete fallback");
            
            if from.is_dir() {
                // Recursive directory copy
                copy_dir_recursive(from, to).await?;
            } else {
                // Simple file copy
                fs::copy(from, to).await.map_err(|e| {
                    LunaError::SystemOperation(format!(
                        "Failed to copy {:?} to {:?}: {}",
                        from, to, e
                    ))
                })?;
            }
            
            // Delete source after successful copy
            if from.is_dir() {
                fs::remove_dir_all(from).await.map_err(|e| {
                    LunaError::SystemOperation(format!(
                        "Failed to remove source directory {:?}: {}",
                        from, e
                    ))
                })?;
            } else {
                fs::remove_file(from).await.map_err(|e| {
                    LunaError::SystemOperation(format!(
                        "Failed to remove source file {:?}: {}",
                        from, e
                    ))
                })?;
            }
            
            info!("✅ Moved {:?} to {:?} (cross-device)", from, to);
            Ok(())
        }
        Err(e) => Err(LunaError::SystemOperation(format!(
            "Failed to move {:?} to {:?}: {}",
            from, to, e
        ))),
    }
}

/// Recursively copy a directory
fn copy_dir_recursive<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst).await?;
        
        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if file_type.is_dir() {
                copy_dir_recursive(&src_path, &dst_path).await?;
            } else {
                fs::copy(&src_path, &dst_path).await?;
            }
        }
        
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[tokio::test]
    async fn test_create_directory_new() {
        let temp = tempdir().unwrap();
        let new_dir = temp.path().join("new_directory");
        
        assert!(!new_dir.exists());
        
        let result = create_directory(&new_dir).await;
        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }
    
    #[tokio::test]
    async fn test_create_directory_idempotent() {
        let temp = tempdir().unwrap();
        let new_dir = temp.path().join("existing_directory");
        
        // Create directory first time
        let result1 = create_directory(&new_dir).await;
        assert!(result1.is_ok());
        
        // Create same directory again - should succeed (idempotent)
        let result2 = create_directory(&new_dir).await;
        assert!(result2.is_ok());
        assert!(new_dir.exists());
    }
    
    #[tokio::test]
    async fn test_create_directory_nested() {
        let temp = tempdir().unwrap();
        let nested_dir = temp.path().join("level1").join("level2").join("level3");
        
        assert!(!nested_dir.exists());
        
        let result = create_directory(&nested_dir).await;
        assert!(result.is_ok());
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());
    }
    
    #[tokio::test]
    async fn test_create_directory_fails_if_file_exists() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("regular_file");
        
        // Create a regular file
        fs::write(&file_path, b"content").unwrap();
        assert!(file_path.exists());
        assert!(file_path.is_file());
        
        // Try to create directory with same path - should fail
        let result = create_directory(&file_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }
    
    #[tokio::test]
    async fn test_move_file_simple() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("destination.txt");
        
        // Create source file
        tokio::fs::write(&src, b"test content").await.unwrap();
        assert!(src.exists());
        
        // Move file
        let result = move_file(&src, &dst).await;
        assert!(result.is_ok());
        
        // Verify move
        assert!(!src.exists());
        assert!(dst.exists());
        
        let content = tokio::fs::read_to_string(&dst).await.unwrap();
        assert_eq!(content, "test content");
    }
    
    #[tokio::test]
    async fn test_move_file_directory() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("source_dir");
        let dst_dir = temp.path().join("destination_dir");
        
        // Create source directory with files
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("file1.txt"), b"content1").await.unwrap();
        tokio::fs::write(src_dir.join("file2.txt"), b"content2").await.unwrap();
        
        // Move directory
        let result = move_file(&src_dir, &dst_dir).await;
        assert!(result.is_ok());
        
        // Verify move
        assert!(!src_dir.exists());
        assert!(dst_dir.exists());
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("file2.txt").exists());
    }
    
    #[tokio::test]
    async fn test_move_file_creates_parent_directory() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("new_dir").join("destination.txt");
        
        // Create source file
        tokio::fs::write(&src, b"test content").await.unwrap();
        
        // Destination parent doesn't exist
        assert!(!dst.parent().unwrap().exists());
        
        // Move file - should create parent directory
        let result = move_file(&src, &dst).await;
        assert!(result.is_ok());
        
        // Verify
        assert!(!src.exists());
        assert!(dst.exists());
        assert!(dst.parent().unwrap().exists());
    }
    
    #[tokio::test]
    async fn test_move_file_fails_if_source_missing() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("nonexistent.txt");
        let dst = temp.path().join("destination.txt");
        
        assert!(!src.exists());
        
        let result = move_file(&src, &dst).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    
    #[tokio::test]
    async fn test_move_file_fails_if_destination_exists() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("destination.txt");
        
        // Create both files
        tokio::fs::write(&src, b"source content").await.unwrap();
        tokio::fs::write(&dst, b"existing content").await.unwrap();
        
        let result = move_file(&src, &dst).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
    
    #[tokio::test]
    async fn test_move_file_nested_directory() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("source");
        let dst_dir = temp.path().join("destination");
        
        // Create nested structure
        tokio::fs::create_dir_all(src_dir.join("sub1").join("sub2")).await.unwrap();
        tokio::fs::write(src_dir.join("root.txt"), b"root").await.unwrap();
        tokio::fs::write(src_dir.join("sub1").join("file1.txt"), b"file1").await.unwrap();
        tokio::fs::write(src_dir.join("sub1").join("sub2").join("file2.txt"), b"file2").await.unwrap();
        
        // Move directory
        let result = move_file(&src_dir, &dst_dir).await;
        assert!(result.is_ok());
        
        // Verify structure is preserved
        assert!(!src_dir.exists());
        assert!(dst_dir.exists());
        assert!(dst_dir.join("root.txt").exists());
        assert!(dst_dir.join("sub1").join("file1.txt").exists());
        assert!(dst_dir.join("sub1").join("sub2").join("file2.txt").exists());
        
        // Verify content
        let content = tokio::fs::read_to_string(dst_dir.join("sub1").join("sub2").join("file2.txt"))
            .await.unwrap();
        assert_eq!(content, "file2");
    }
    
    #[tokio::test]
    async fn test_move_file_preserves_content() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("destination.txt");
        
        let test_content = "This is a test file with some content.\nLine 2\nLine 3";
        tokio::fs::write(&src, test_content).await.unwrap();
        
        move_file(&src, &dst).await.unwrap();
        
        let read_content = tokio::fs::read_to_string(&dst).await.unwrap();
        assert_eq!(read_content, test_content);
    }
    
    #[tokio::test]
    async fn test_move_file_empty_file() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("empty.txt");
        let dst = temp.path().join("moved_empty.txt");
        
        // Create empty file
        tokio::fs::write(&src, b"").await.unwrap();
        
        let result = move_file(&src, &dst).await;
        assert!(result.is_ok());
        
        assert!(!src.exists());
        assert!(dst.exists());
        
        let metadata = tokio::fs::metadata(&dst).await.unwrap();
        assert_eq!(metadata.len(), 0);
    }
    
    #[tokio::test]
    async fn test_create_directory_with_unicode_name() {
        let temp = tempdir().unwrap();
        let unicode_dir = temp.path().join("测试目录_тест_🚀");
        
        let result = create_directory(&unicode_dir).await;
        assert!(result.is_ok());
        assert!(unicode_dir.exists());
    }
    
    #[tokio::test]
    async fn test_move_file_with_unicode_name() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("файл_源.txt");
        let dst = temp.path().join("destination_文件.txt");
        
        tokio::fs::write(&src, "Unicode content: 你好世界").await.unwrap();
        
        let result = move_file(&src, &dst).await;
        assert!(result.is_ok());
        
        assert!(!src.exists());
        assert!(dst.exists());
        
        let content = tokio::fs::read_to_string(&dst).await.unwrap();
        assert!(content.contains("你好世界"));
    }
}
