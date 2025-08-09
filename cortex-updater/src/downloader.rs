use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use reqwest;
use sha2::{Sha256, Digest};
use tokio::io::AsyncWriteExt;

/// Download manager for updates
pub struct Downloader {
    download_dir: PathBuf,
    client: reqwest::Client,
}

impl Downloader {
    pub fn new(download_dir: PathBuf) -> Self {
        Self {
            download_dir,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
        }
    }
    
    /// Download a file with progress callback
    pub async fn download(
        &self,
        url: &str,
        expected_sha256: &str,
        progress_callback: impl Fn(u64, u64) + Send + 'static,
    ) -> Result<PathBuf> {
        // Extract filename from URL
        let filename = url.split('/').last()
            .context("Invalid download URL")?;
        
        let download_path = self.download_dir.join(filename);
        let temp_path = self.download_dir.join(format!("{}.tmp", filename));
        
        // Start download
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to start download")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }
        
        let total_size = response
            .content_length()
            .unwrap_or(0);
        
        // Create temp file
        let mut file = tokio::fs::File::create(&temp_path).await
            .context("Failed to create download file")?;
        
        // Download with progress
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut hasher = Sha256::new();
        
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Download interrupted")?;
            
            // Write to file
            file.write_all(&chunk).await
                .context("Failed to write download data")?;
            
            // Update hash
            hasher.update(&chunk);
            
            // Update progress
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        
        file.flush().await?;
        drop(file);
        
        // Verify checksum
        let result = hasher.finalize();
        let file_hash = hex::encode(result);
        
        if file_hash != expected_sha256 {
            // Remove corrupt download
            fs::remove_file(&temp_path)?;
            anyhow::bail!(
                "Checksum verification failed. Expected: {}, Got: {}",
                expected_sha256, file_hash
            );
        }
        
        // Move to final location
        fs::rename(&temp_path, &download_path)?;
        
        Ok(download_path)
    }
    
    /// Resume a partial download
    pub async fn resume_download(
        &self,
        url: &str,
        partial_path: &Path,
        expected_sha256: &str,
        progress_callback: impl Fn(u64, u64) + Send + 'static,
    ) -> Result<PathBuf> {
        let existing_size = fs::metadata(partial_path)?.len();
        
        // Request with range header
        let response = self.client
            .get(url)
            .header("Range", format!("bytes={}-", existing_size))
            .send()
            .await?;
        
        if response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            // Server doesn't support resume, start fresh
            fs::remove_file(partial_path)?;
            return self.download(url, expected_sha256, progress_callback).await;
        }
        
        if !response.status().is_success() {
            anyhow::bail!("Resume failed with status: {}", response.status());
        }
        
        // Continue download
        let total_size = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split('/').last())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(existing_size);
        
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(partial_path)
            .await?;
        
        let mut downloaded = existing_size;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        
        file.flush().await?;
        
        // Verify complete file
        let mut file = fs::File::open(partial_path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let result = hasher.finalize();
        let file_hash = hex::encode(result);
        
        if file_hash != expected_sha256 {
            fs::remove_file(partial_path)?;
            anyhow::bail!("Checksum verification failed after resume");
        }
        
        Ok(partial_path.to_path_buf())
    }
    
    /// Clean up partial downloads
    pub fn cleanup_partial_downloads(&self) -> Result<()> {
        let entries = fs::read_dir(&self.download_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "tmp" {
                    fs::remove_file(&path)?;
                }
            }
        }
        
        Ok(())
    }
}