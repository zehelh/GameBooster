//! # WinDivert Wrapper
//!
//! A custom wrapper to dynamically load WinDivert.dll and call its functions.
//! This avoids the build script issues from the `windivert-sys` crate.

use anyhow::{anyhow, Result};
use std::ffi::c_void;
use std::mem;
use windows_sys::Win32::Foundation::{GetLastError, TRUE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct WinDivertAddress {
    pub timestamp: i64,
    pub layer: u32,
    pub event: u32,
    pub sniffed: u32,
    pub outbound: u32,
    pub loopback: u32,
    pub impostor: u32,
    pub ipv6: u32,
    pub ip_checksum: u32,
    pub tcp_checksum: u32,
    pub udp_checksum: u32,
    pub process_id: u32,
    // The actual struct in C contains a union here of if_idx/sub_if_idx
    // For this wrapper, we don't need it, so we can represent it as padding.
    _padding: [u8; 8], 
}

impl WinDivertAddress {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn process_id(&self) -> Option<u32> {
        if self.process_id != 0 { Some(self.process_id) } else { None }
    }
}

type WinDivertOpen = unsafe extern "system" fn(
    filter: *const i8,
    layer: u32,
    priority: i16,
    flags: u64,
) -> HANDLE;
type WinDivertRecv = unsafe extern "system" fn(
    handle: HANDLE,
    p_packet: *mut c_void,
    packet_len: u32,
    p_addr: *mut WinDivertAddress,
    recv_len: *mut u32,
) -> i32;
type WinDivertSend = unsafe extern "system" fn(
    handle: HANDLE,
    p_packet: *const c_void,
    packet_len: u32,
    p_addr: *const WinDivertAddress,
    send_len: *mut u32,
) -> i32;
type WinDivertClose = unsafe extern "system" fn(handle: HANDLE) -> i32;
type FreeLibrary = unsafe extern "system" fn(h_lib_module: HANDLE) -> i32;

#[derive(Clone)]
struct WinDivertFns {
    win_divert_open: WinDivertOpen,
    win_divert_recv: WinDivertRecv,
    win_divert_send: WinDivertSend,
    win_divert_close: WinDivertClose,
    free_library: FreeLibrary,
}

#[derive(Clone)]
pub struct WinDivert {
    lib: HANDLE,
    fns: WinDivertFns,
    pub handle: HANDLE,
}

impl WinDivert {
    pub fn new() -> Result<Self> {
        unsafe {
            let lib = LoadLibraryA("WinDivert.dll\0".as_ptr());
            if lib == 0 {
                return Err(anyhow!(
                    "WinDivert.dll not found. Make sure it's in the same directory as the executable."
                ));
            }

            let kernel32 = LoadLibraryA(b"kernel32.dll\0".as_ptr());
            if kernel32 == 0 {
                return Err(anyhow!("Failed to load kernel32.dll"));
            }

            macro_rules! get_fn {
                ($lib:expr, $name:ident) => {
                    match GetProcAddress($lib, stringify!($name).as_bytes().as_ptr()) {
                        Some(f) => mem::transmute::<_, $name>(f),
                        None => {
                            (mem::transmute::<_, FreeLibrary>(GetProcAddress(kernel32, b"FreeLibrary\0".as_ptr()).unwrap()))(lib);
                            return Err(anyhow!(format!("Failed to get function {}", stringify!($name))))
                        },
                    }
                };
            }

            let fns = WinDivertFns {
                win_divert_open: get_fn!(lib, WinDivertOpen),
                win_divert_recv: get_fn!(lib, WinDivertRecv),
                win_divert_send: get_fn!(lib, WinDivertSend),
                win_divert_close: get_fn!(lib, WinDivertClose),
                free_library: get_fn!(kernel32, FreeLibrary),
            };
            
            let handle = (fns.win_divert_open)(b"true\0".as_ptr() as *const i8, 0, 0, 0);

            if handle == INVALID_HANDLE_VALUE {
                (fns.free_library)(lib);
                Err(anyhow!("WinDivertOpen failed with error {}", GetLastError()))
            } else {
                Ok(Self { lib, fns, handle })
            }
        }
    }

    pub fn recv<'a>(&self, packet: &'a mut [u8], addr_opt: Option<&mut WinDivertAddress>) -> Result<(&'a [u8], WinDivertAddress), String> {
        let mut recv_len: u32 = 0;
        let mut temp_addr = WinDivertAddress::new();
        let addr_ptr = &mut temp_addr as *mut _;
        
        let success = unsafe {
            (self.fns.win_divert_recv)(
                self.handle,
                packet.as_mut_ptr() as *mut c_void,
                packet.len() as u32,
                addr_ptr,
                &mut recv_len,
            ) == TRUE
        };

        if success {
            if let Some(out_addr) = addr_opt {
                *out_addr = temp_addr;
            }
            Ok((&packet[..recv_len as usize], temp_addr))
        } else {
            Err(format!("WinDivertRecv failed with error {}", unsafe { GetLastError() }))
        }
    }

    pub fn send(&self, packet: &[u8], addr: &WinDivertAddress) -> Result<usize, String> {
        let mut sent_len: u32 = 0;
        if unsafe { (self.fns.win_divert_send)(self.handle, packet.as_ptr() as *const c_void, packet.len() as u32, addr, &mut sent_len) } == TRUE {
            Ok(sent_len as usize)
        } else {
            Err(format!("WinDivertSend failed with error {}", unsafe { GetLastError() }))
        }
    }

    pub fn close(&mut self) {
        unsafe {
            if self.lib != 0 {
                if self.handle != INVALID_HANDLE_VALUE {
                    (self.fns.win_divert_close)(self.handle);
                }
                (self.fns.free_library)(self.lib);
                self.lib = 0;
            }
        }
    }
} 