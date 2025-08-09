use std::env;

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformInfo {
    pub os: super::Platform,
    pub version: String,
    pub arch: String,
    pub desktop_environment: Option<String>,
}

impl PlatformInfo {
    pub fn current() -> Self {
        let os = super::Platform::current();
        let arch = env::consts::ARCH.to_string();
        
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winnt::OSVERSIONINFOEXW;
            use winapi::um::sysinfoapi::GetVersionExW;
            use std::mem;
            
            let mut version_info: OSVERSIONINFOEXW = unsafe { mem::zeroed() };
            version_info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEXW>() as u32;
            
            let version = unsafe {
                if GetVersionExW(&mut version_info as *mut _ as *mut _) != 0 {
                    format!("{}.{}.{}", 
                        version_info.dwMajorVersion,
                        version_info.dwMinorVersion,
                        version_info.dwBuildNumber)
                } else {
                    "Unknown".to_string()
                }
            };
            
            Self {
                os,
                version,
                arch,
                desktop_environment: None,
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let version = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            Self {
                os,
                version,
                arch,
                desktop_environment: None,
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let version = std::fs::read_to_string("/proc/version")
                .ok()
                .and_then(|s| s.split_whitespace().nth(2).map(|v| v.to_string()))
                .unwrap_or_else(|| "Unknown".to_string());
            
            let desktop_environment = env::var("XDG_CURRENT_DESKTOP")
                .ok()
                .or_else(|| env::var("DESKTOP_SESSION").ok());
            
            Self {
                os,
                version,
                arch,
                desktop_environment,
            }
        }
    }
    
    pub fn supports_trash(&self) -> bool {
        true
    }
    
    pub fn supports_clipboard(&self) -> bool {
        match self.os {
            super::Platform::Linux => {
                env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok()
            }
            _ => true,
        }
    }
    
    pub fn home_directory() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            env::var("USERPROFILE").ok()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            env::var("HOME").ok()
        }
    }
    
    pub fn config_directory() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            env::var("APPDATA").ok().map(|p| format!("{}\\cortex", p))
        }
        
        #[cfg(target_os = "macos")]
        {
            Self::home_directory().map(|h| format!("{}/Library/Application Support/cortex", h))
        }
        
        #[cfg(target_os = "linux")]
        {
            env::var("XDG_CONFIG_HOME")
                .ok()
                .or_else(|| Self::home_directory().map(|h| format!("{}/.config", h)))
                .map(|p| format!("{}/cortex", p))
        }
    }
}