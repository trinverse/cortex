use crate::{TrashItem, TrashOperations};
use anyhow::Result;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use winapi::um::shellapi::{
    SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FO_DELETE, SHFILEOPSTRUCTW,
};
use winapi::um::winnt::WCHAR;

pub struct WindowsTrash {}

impl WindowsTrash {
    pub fn new() -> Self {
        Self {}
    }

    fn to_wide_string(s: &OsStr) -> Vec<WCHAR> {
        let mut vec: Vec<WCHAR> = s.encode_wide().collect();
        vec.push(0); // null terminator
        vec.push(0); // double null terminator for SHFileOperation
        vec
    }
}

impl TrashOperations for WindowsTrash {
    fn move_to_trash(&self, path: &Path) -> Result<()> {
        let wide_path = Self::to_wide_string(path.as_os_str());

        let mut file_op = SHFILEOPSTRUCTW {
            hwnd: std::ptr::null_mut(),
            wFunc: FO_DELETE as u32,
            pFrom: wide_path.as_ptr(),
            pTo: std::ptr::null(),
            fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION) as u16,
            fAnyOperationsAborted: 0,
            hNameMappings: std::ptr::null_mut(),
            lpszProgressTitle: std::ptr::null(),
        };

        let result = unsafe { SHFileOperationW(&mut file_op) };

        if result != 0 {
            return Err(anyhow::anyhow!(
                "Failed to move to recycle bin: error code {}",
                result
            ));
        }

        Ok(())
    }

    fn restore_from_trash(&self, _path: &Path) -> Result<()> {
        // Windows doesn't provide a simple API for restoring from recycle bin
        // This would require COM interfaces and is quite complex
        Err(anyhow::anyhow!(
            "Restore from recycle bin is not yet implemented on Windows"
        ))
    }

    fn empty_trash(&self) -> Result<()> {
        // This requires SHEmptyRecycleBin API
        use winapi::um::shellapi::SHEmptyRecycleBinW;
        use winapi::um::winuser::HWND_DESKTOP;

        let result = unsafe { SHEmptyRecycleBinW(HWND_DESKTOP, std::ptr::null(), 0) };

        if result != 0 {
            return Err(anyhow::anyhow!(
                "Failed to empty recycle bin: error code {}",
                result
            ));
        }

        Ok(())
    }

    fn list_trash_contents(&self) -> Result<Vec<TrashItem>> {
        // Listing recycle bin contents requires COM interfaces
        // For now, return empty list
        Ok(Vec::new())
    }
}
