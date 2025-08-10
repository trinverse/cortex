use crate::{TrashItem, TrashOperations};
use anyhow::Result;
use cocoa::foundation::NSAutoreleasePool;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use objc_foundation::{INSString, NSString};
use std::path::Path;

#[derive(Default)]
pub struct MacOSTrash {}


impl MacOSTrash {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TrashOperations for MacOSTrash {
    fn move_to_trash(&self, path: &Path) -> Result<()> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());

            let file_manager: *mut Object = msg_send![class!(NSFileManager), defaultManager];
            let path_str = NSString::from_str(&path.to_string_lossy());
            let url: *mut Object = msg_send![class!(NSURL), fileURLWithPath: path_str];

            let mut error: *mut Object = std::ptr::null_mut();
            let result: bool = msg_send![
                file_manager,
                trashItemAtURL: url
                resultingItemURL: std::ptr::null_mut::<*mut Object>()
                error: &mut error
            ];

            if !result {
                return Err(anyhow::anyhow!("Failed to move item to trash"));
            }

            Ok(())
        }
    }

    fn restore_from_trash(&self, _path: &Path) -> Result<()> {
        // macOS doesn't provide a simple API for restoring from trash
        // Would need to track original locations separately
        Err(anyhow::anyhow!(
            "Restore from trash is not yet implemented on macOS"
        ))
    }

    fn empty_trash(&self) -> Result<()> {
        unsafe {
            let _pool = NSAutoreleasePool::new(std::ptr::null_mut());

            let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
            let _: () = msg_send![workspace, emptyTrash];

            Ok(())
        }
    }

    fn list_trash_contents(&self) -> Result<Vec<TrashItem>> {
        // Listing trash contents on macOS is complex and requires
        // accessing ~/.Trash directory with proper permissions
        Ok(Vec::new())
    }
}
