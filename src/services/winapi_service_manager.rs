// Windows Service Manager using WinAPI directly
// Manages Windows services without spawning PowerShell processes

use anyhow::{Result};
use anyhow::anyhow;

#[cfg(target_os = "windows")]
use std::ffi::CString;
#[cfg(target_os = "windows")]
use std::ptr;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerA, OpenServiceA, QueryServiceStatus,
    SC_MANAGER_ALL_ACCESS, SERVICE_QUERY_STATUS, SERVICE_STATUS, SERVICE_STOPPED,
    SERVICE_START_PENDING, SERVICE_STOP_PENDING, SERVICE_RUNNING, SERVICE_CONTINUE_PENDING,
    SERVICE_PAUSE_PENDING, SERVICE_PAUSED, SC_HANDLE,
};

pub struct ServiceManager;

#[cfg(target_os = "windows")]
impl ServiceManager {
    /// Open service control manager with appropriate permissions
    fn open_scm() -> Result<SC_HANDLE> {
        unsafe {
            let scm_handle = OpenSCManagerA(
                ptr::null(),
                ptr::null(),
                SC_MANAGER_ALL_ACCESS,
            );
            if scm_handle == std::ptr::null_mut() {
                Err(anyhow!("Could not open SCM"))
            } else {
                Ok(scm_handle)
            }
        }
    }

    /// Open a specific service
    fn open_service(scm_handle: SC_HANDLE, service_name: &str, access: u32) -> Result<SC_HANDLE> {
        let service_name_c = CString::new(service_name).unwrap();
        unsafe {
            let service_handle = OpenServiceA(
                scm_handle,
                service_name_c.as_ptr() as *const u8,
                access,
            );
            if service_handle == std::ptr::null_mut() {
                Err(anyhow!("Could not open service {}", service_name))
            } else {
                Ok(service_handle)
            }
        }
    }

    /// Get service status
    pub fn get_service_status(service_name: &str) -> Result<String> {
        let scm_handle = Self::open_scm()?;
        
        let service_handle = match Self::open_service(scm_handle, service_name, SERVICE_QUERY_STATUS) {
            Ok(handle) => handle,
            Err(_) => {
                unsafe { CloseServiceHandle(scm_handle) };
                return Ok("Not Found".to_string());
            }
        };

        let mut status = SERVICE_STATUS {
            dwServiceType: 0,
            dwCurrentState: 0,
            dwControlsAccepted: 0,
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };

        let result = unsafe {
            QueryServiceStatus(service_handle, &mut status)
        };

        unsafe {
            CloseServiceHandle(service_handle);
            CloseServiceHandle(scm_handle);
        }

        if result == 0 {
            return Err(anyhow!("Failed to query service status"));
        }

        let status_str = match status.dwCurrentState {
            SERVICE_STOPPED => "Stopped",
            SERVICE_START_PENDING => "Starting",
            SERVICE_STOP_PENDING => "Stopping",
            SERVICE_RUNNING => "Running",
            SERVICE_CONTINUE_PENDING => "Resuming",
            SERVICE_PAUSE_PENDING => "Pausing",
            SERVICE_PAUSED => "Paused",
            _ => "Unknown",
        };

        Ok(status_str.to_string())
    }

    #[cfg(target_os = "windows")]
    pub fn is_service_running(_service_name: &str) -> Result<bool> {
        // TODO: Implement actual check if service is running
        Ok(false) // Placeholder
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_service_running(_service_name: &str) -> Result<bool> {
        Ok(false) // Placeholder for non-Windows
    }
}
