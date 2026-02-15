//! 系统状态处理器
//!
//! 获取服务器系统状态信息

use crate::auth::AdminOnly;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use sysinfo::{Disks, System};

/// 系统状态响应
#[derive(Serialize)]
pub struct SystemStatsResponse {
    pub status: String,
    pub server_ip: String,
    pub uptime_seconds: u64,
    pub cpu_usage: f64,
    pub memory: MemoryStats,
    pub disk: DiskStats,
    pub network: NetworkStats,
}

#[derive(Serialize)]
pub struct MemoryStats {
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub used_percent: f64,
}

#[derive(Serialize)]
pub struct DiskStats {
    pub total_gb: u64,
    pub used_gb: u64,
    pub free_gb: u64,
    pub used_percent: f64,
}

#[derive(Serialize)]
pub struct NetworkStats {
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

/// 获取系统状态（仅管理员可访问）
pub async fn get_system_stats(
    _admin: AdminOnly,
    State(state): State<Arc<AppState>>,
) -> Json<SystemStatsResponse> {
    let uptime = get_uptime();
    let memory = get_memory_stats();
    let disk = get_disk_stats();
    let cpu = get_cpu_usage();
    let network = get_network_stats();

    let status = if uptime > 0 { "online" } else { "offline" };

    // 获取服务器 IP
    let server_ip = get_server_ip(&state.config.server.host, state.config.server.port);

    Json(SystemStatsResponse {
        status: status.to_string(),
        server_ip,
        uptime_seconds: uptime,
        cpu_usage: cpu,
        memory,
        disk,
        network,
    })
}

/// 获取服务器 IP 地址
fn get_server_ip(host: &str, port: u16) -> String {
    // 如果 host 是 0.0.0.0，尝试获取本机 IP
    if host == "0.0.0.0" {
        // 尝试通过连接来确定实际 IP
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return addr.ip().to_string();
                }
            }
        }
        // 回退到 localhost
        return "127.0.0.1".to_string();
    }

    // 如果 host 是域名或 IP，尝试解析
    let addr_string = format!("{host}:{port}");
    if let Ok(mut addrs) = addr_string.as_str().to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            return addr.ip().to_string();
        }
    }

    host.to_string()
}

/// 获取系统运行时间（秒）
fn get_uptime() -> u64 {
    // sysinfo 0.33+ 使用平台特定方式获取 uptime
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
            if let Some(uptime_str) = content.split_whitespace().next() {
                return uptime_str.parse::<u64>().unwrap_or(0);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("kern.boottime").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // 解析 kern.boottime 输出
            if let Some(start) = output_str.find("sec = ") {
                let num_start = start + 6;
                if let Some(end) = output_str[num_start..].find(',') {
                    if let Ok(boot_sec) = output_str[num_start..num_start + end].parse::<i64>() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        return (now - boot_sec) as u64;
                    }
                }
            }
        }
    }
    0
}

/// 获取内存使用情况
fn get_memory_stats() -> MemoryStats {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.available_memory();
    let used_percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    MemoryStats {
        total_kb: total / 1024,
        used_kb: used / 1024,
        free_kb: free / 1024,
        used_percent,
    }
}

/// 获取磁盘使用情况
fn get_disk_stats() -> DiskStats {
    let disks = Disks::new_with_refreshed_list();

    // 获取第一个磁盘（通常是系统磁盘）
    if let Some(disk) = disks.list().first() {
        let total = disk.total_space();
        let free = disk.available_space();
        let used = total.saturating_sub(free);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        DiskStats {
            total_gb: total / (1024 * 1024 * 1024),
            used_gb: used / (1024 * 1024 * 1024),
            free_gb: free / (1024 * 1024 * 1024),
            used_percent,
        }
    } else {
        DiskStats {
            total_gb: 0,
            used_gb: 0,
            free_gb: 0,
            used_percent: 0.0,
        }
    }
}

/// 获取CPU使用率
fn get_cpu_usage() -> f64 {
    let mut sys = System::new_all();
    // 需要刷新两次以获取CPU使用率
    sys.refresh_cpu_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    if cpus.is_empty() {
        return 0.0;
    }

    let total_usage: f64 =
        cpus.iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpus.len() as f64;
    total_usage
}

/// 获取网络流量（简化实现，返回0）
fn get_network_stats() -> NetworkStats {
    // 网络流量统计在跨平台实现较为复杂，这里简化处理
    NetworkStats {
        upload_bytes: 0,
        download_bytes: 0,
    }
}
