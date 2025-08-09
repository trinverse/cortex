use std::path::Path;
use anyhow::{Result, Context};
use crate::ClipboardOperations;
use winapi::um::winuser::{
    OpenClipboard, CloseClipboard, EmptyClipboard, SetClipboardData, GetClipboardData,
    CF_UNICODETEXT, CF_HDROP
};
use winapi::um::winnls::{MultiByteToWideChar, WideCharToMultiByte, CP_UTF8};
use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub struct WindowsClipboard {}

impl WindowsClipboard {
    pub fn new() -> Self {
        Self {}
    }
    
    fn to_wide_string(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }
}

impl ClipboardOperations for WindowsClipboard {
    fn copy_text(&self, text: &str) -> Result<()> {
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return Err(anyhow::anyhow!("Failed to open clipboard"));
            }
            
            let result = (|| {
                if EmptyClipboard() == 0 {
                    return Err(anyhow::anyhow!("Failed to empty clipboard"));
                }
                
                let wide_text = Self::to_wide_string(text);
                let size = wide_text.len() * std::mem::size_of::<u16>();
                
                let h_global = GlobalAlloc(GMEM_MOVEABLE, size);
                if h_global.is_null() {
                    return Err(anyhow::anyhow!("Failed to allocate global memory"));
                }
                
                let ptr = GlobalLock(h_global) as *mut u16;
                if ptr.is_null() {
                    return Err(anyhow::anyhow!("Failed to lock global memory"));
                }
                
                ptr::copy_nonoverlapping(wide_text.as_ptr(), ptr, wide_text.len());
                GlobalUnlock(h_global);
                
                if SetClipboardData(CF_UNICODETEXT, h_global).is_null() {
                    return Err(anyhow::anyhow!("Failed to set clipboard data"));
                }
                
                Ok(())
            })();
            
            CloseClipboard();
            result
        }
    }
    
    fn paste_text(&self) -> Result<String> {
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return Err(anyhow::anyhow!("Failed to open clipboard"));
            }
            
            let result = (|| {
                let h_data = GetClipboardData(CF_UNICODETEXT);
                if h_data.is_null() {
                    return Err(anyhow::anyhow!("No text data in clipboard"));
                }
                
                let ptr = GlobalLock(h_data) as *const u16;
                if ptr.is_null() {
                    return Err(anyhow::anyhow!("Failed to lock clipboard data"));
                }
                
                let mut len = 0;
                while *ptr.offset(len) != 0 {
                    len += 1;
                }
                
                let slice = std::slice::from_raw_parts(ptr, len as usize);
                let result = String::from_utf16_lossy(slice);
                
                GlobalUnlock(h_data);
                Ok(result)
            })();
            
            CloseClipboard();
            result
        }
    }
    
    fn copy_files(&self, _paths: &[&Path]) -> Result<()> {
        // File copying to clipboard on Windows requires CF_HDROP format
        // This is more complex and would require DROPFILES structure
        Err(anyhow::anyhow!("File clipboard operations not yet implemented on Windows"))
    }
    
    fn paste_files(&self) -> Result<Vec<String>> {
        // File pasting from clipboard on Windows requires CF_HDROP format
        Err(anyhow::anyhow!("File clipboard operations not yet implemented on Windows"))
    }
    
    fn has_content(&self) -> bool {
        true // Windows clipboard is always available
    }
}