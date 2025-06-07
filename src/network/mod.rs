//! # Network Module
//!
//! This module handles network-related functionalities, using a custom WinDivert wrapper
//! to monitor and block network traffic from specific processes.

pub mod process_monitor;
mod windivert_wrapper;

use anyhow::{Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use self::windivert_wrapper::{WinDivert};
use tracing::{error, info};
use crate::network::process_monitor::scan_network_processes as scan_processes;

// Re-defining constants that were in the original crate's prelude
const WINDIVERT_LAYER_NETWORK: i32 = 0;

/// Represents a process with its network activity status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkProcess {
    pub pid: u32,
    pub name: String,
    pub is_blocked: bool,
}

/// The main orchestrator for network operations using WinDivert.
#[derive(Clone)]
pub struct NetworkManager {
    blocked_pids: Arc<Mutex<HashSet<u32>>>,
    stop_flag: Arc<Mutex<bool>>,
    pub processes: Vec<NetworkProcess>,
}

impl NetworkManager {
    /// Creates a new NetworkManager and starts the packet filtering thread.
    pub fn new() -> Result<Self, String> {
        let manager = Self {
            blocked_pids: Arc::new(Mutex::new(HashSet::new())),
            stop_flag: Arc::new(Mutex::new(false)),
            processes: Vec::new(),
        };
        manager.start_filter_thread();
        Ok(manager)
    }

    fn start_filter_thread(&self) {
        let blocked_pids_clone = self.blocked_pids.clone();
        let stop_flag_clone = self.stop_flag.clone();

        thread::spawn(move || {
            let windivert = match WinDivert::new() {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to open WinDivert handle: {}", e);
                    return;
                }
            };
            
            info!("WinDivert filter thread started successfully.");

            let mut packet = [0u8; 8192];

            while !*stop_flag_clone.lock().unwrap() {
                if let Ok((packet_slice, addr)) = windivert.recv(&mut packet, None) {
                    if let Some(pid) = addr.process_id() {
                        let blocked = blocked_pids_clone.lock().unwrap().contains(&pid);
                        if !blocked {
                            if let Err(e) = windivert.send(packet_slice, &addr) {
                                error!("Failed to resend packet: {}", e);
                            }
                        }
                    } else {
                         if let Err(e) = windivert.send(packet_slice, &addr) {
                            error!("Failed to resend packet (no PID): {}", e);
                        }
                    }
                }
            }
            info!("WinDivert filter thread stopped.");
        });
    }

    pub async fn scan_network_processes(&mut self) -> Result<(), anyhow::Error> {
        let process_infos = scan_processes().await?;
        let blocked_pids = self.blocked_pids.lock().unwrap();
        self.processes = process_infos
            .into_iter()
            .map(|info| NetworkProcess {
                pid: info.pid,
                name: info.name,
                is_blocked: blocked_pids.contains(&info.pid),
            })
            .collect();
        Ok(())
    }

    /// Updates the blocked status of a process.
    /// This is now a simple, synchronous operation.
    pub fn set_process_blocked(&self, pid: u32, blocked: bool) {
        let mut blocked_pids = self.blocked_pids.lock().unwrap();
        if blocked {
            blocked_pids.insert(pid);
        } else {
            blocked_pids.remove(&pid);
        }
    }

    pub fn is_pid_blocked(&self, pid: u32) -> bool {
        self.blocked_pids.lock().unwrap().contains(&pid)
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        *self.stop_flag.lock().unwrap() = true;
    }
}
