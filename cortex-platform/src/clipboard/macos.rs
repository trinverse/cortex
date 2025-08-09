use std::path::Path;
use anyhow::{Result, Context};
use crate::ClipboardOperations;
use objc::{msg_send, sel, sel_impl};
use objc::runtime::Object;
use objc_foundation::{NSString, INSString, NSArray, INSArray};
use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
use cocoa::foundation::NSAutoreleasePool;

pub struct MacOSClipboard {}

impl MacOSClipboard {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClipboardOperations for MacOSClipboard {
    fn copy_text(&self, text: &str) -> Result<()> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());
            
            let pasteboard = NSPasteboard::generalPasteboard(std::ptr::null_mut());
            let ns_string = NSString::from_str(text);
            
            let _: () = msg_send![pasteboard, clearContents];
            let success: bool = msg_send![
                pasteboard,
                setString: ns_string
                forType: NSPasteboardTypeString
            ];
            
            if !success {
                return Err(anyhow::anyhow!("Failed to copy text to clipboard"));
            }
            
            Ok(())
        }
    }
    
    fn paste_text(&self) -> Result<String> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());
            
            let pasteboard = NSPasteboard::generalPasteboard(std::ptr::null_mut());
            let ns_string: *mut Object = msg_send![
                pasteboard,
                stringForType: NSPasteboardTypeString
            ];
            
            if ns_string.is_null() {
                return Err(anyhow::anyhow!("No text data in clipboard"));
            }
            
            let text = NSString::from_ptr(ns_string).as_str().to_string();
            Ok(text)
        }
    }
    
    fn copy_files(&self, paths: &[&Path]) -> Result<()> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());
            
            let pasteboard = NSPasteboard::generalPasteboard(std::ptr::null_mut());
            let _: () = msg_send![pasteboard, clearContents];
            
            let file_urls: Vec<*mut Object> = paths.iter()
                .map(|p| {
                    let path_str = NSString::from_str(&p.to_string_lossy());
                    let url: *mut Object = msg_send![class!(NSURL), fileURLWithPath: path_str];
                    url
                })
                .collect();
            
            let ns_array = NSArray::from_vec(file_urls);
            let success: bool = msg_send![
                pasteboard,
                writeObjects: ns_array
            ];
            
            if !success {
                return Err(anyhow::anyhow!("Failed to copy files to clipboard"));
            }
            
            Ok(())
        }
    }
    
    fn paste_files(&self) -> Result<Vec<String>> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());
            
            let pasteboard = NSPasteboard::generalPasteboard(std::ptr::null_mut());
            let file_url_type = NSString::from_str("public.file-url");
            
            let urls: *mut Object = msg_send![
                pasteboard,
                readObjectsForClasses: NSArray::from_vec(vec![class!(NSURL)])
                options: std::ptr::null::<Object>()
            ];
            
            if urls.is_null() {
                return Ok(Vec::new());
            }
            
            let ns_array = NSArray::from_ptr(urls);
            let count = ns_array.count();
            let mut paths = Vec::new();
            
            for i in 0..count {
                let url = ns_array.object_at(i);
                let path: *mut Object = msg_send![url, path];
                if !path.is_null() {
                    let path_str = NSString::from_ptr(path).as_str().to_string();
                    paths.push(path_str);
                }
            }
            
            Ok(paths)
        }
    }
    
    fn has_content(&self) -> bool {
        true // macOS pasteboard is always available
    }
}