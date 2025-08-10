use crate::ClipboardOperations;
use anyhow::Result;
use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
use cocoa::foundation::NSAutoreleasePool;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use objc_foundation::{INSString, NSString};
use std::path::Path;

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

            let ns_string_ref: &NSString = &*(ns_string as *const NSString);
            let text = ns_string_ref.as_str().to_string();
            Ok(text)
        }
    }

    fn copy_files(&self, paths: &[&Path]) -> Result<()> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());

            let pasteboard = NSPasteboard::generalPasteboard(std::ptr::null_mut());
            let _: () = msg_send![pasteboard, clearContents];

            let file_urls: Vec<*mut Object> = paths
                .iter()
                .map(|p| {
                    let path_str = NSString::from_str(&p.to_string_lossy());
                    let url: *mut Object = msg_send![class!(NSURL), fileURLWithPath: path_str];
                    url
                })
                .collect();

            // Create NSArray directly from pointers
            let ns_array: *mut Object = msg_send![class!(NSArray), arrayWithObjects:file_urls.as_ptr() count:file_urls.len()];
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
            let _file_url_type = NSString::from_str("public.file-url");

            // Create an NSArray with NSURL class
            let url_class = class!(NSURL);
            let classes_array: *mut Object = msg_send![class!(NSArray), arrayWithObject:url_class];

            let urls: *mut Object = msg_send![
                pasteboard,
                readObjectsForClasses: classes_array
                options: std::ptr::null::<Object>()
            ];

            if urls.is_null() {
                return Ok(Vec::new());
            }

            // Work with NSArray directly via messages
            let count: usize = msg_send![urls, count];
            let mut paths = Vec::new();

            for i in 0..count {
                let url: *mut Object = msg_send![urls, objectAtIndex:i];
                let path: *mut Object = msg_send![url, path];
                if !path.is_null() {
                    let path_ref: &NSString = &*(path as *const NSString);
                    let path_str = path_ref.as_str().to_string();
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
