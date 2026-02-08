//! 系统状态处理器
//!
//! 获取服务器系统状态信息

use axum::{Json, response::IntoResponse};
use serde::Serialize;
use std::fs;

/// 系统状态响应
#[derive(Serialize)]
pub struct SystemStatsResponse {
    pub status: String,
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

/// 获取系统状态
pub async fn get_system_stats() -> Json<SystemStatsResponse> {
    let uptime = get_uptime();
    let memory = get_memory_stats();
    let disk = get_disk_stats();
    let cpu = get_cpu_usage();
    let network = get_network_stats();

    let status = if uptime > 0 { "online" } else { "offline" };

    Json(SystemStatsResponse {
        status: status.to_string(),
        uptime_seconds: uptime,
        cpu_usage: cpu,
        memory,
        disk,
        network,
    })
}

/// 获取系统运行时间（秒）
fn get_uptime() -> u64 {
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_str) = content.split_whitespace().next() {
            return uptime_str.parse::<u64>().unwrap_or(0);
        }
    }
    0
}

/// 获取内存使用情况
fn get_memory_stats() -> MemoryStats {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut available = 0u64;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total = val.parse::<u64>().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") || line.starts_with("MemFree:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    available = val.parse::<u64>().unwrap_or(0);
                }
            }
        }

        let used = total.saturating_sub(available);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        return MemoryStats {
            total_kb: total,
            used_kb: used,
            free_kb: available,
            used_percent,
        };
    }

    MemoryStats {
        total_kb: 0,
        used_kb: 0,
        free_kb: 0,
        used_percent: 0.0,
    }
}

/// 获取磁盘使用情况
fn get_disk_stats() -> DiskStats {
    // 使用 sysinfo 库获取磁盘信息更准确
    // 这里简单实现，返回0因为需要额外依赖
    DiskStats {
        total_gb: 0,
        used_gb: 0,
        free_gb: 0,
        used_percent: 0.0,
    }
}

/// 获取CPU使用率（简单实现）
fn get_cpu_usage() -> f64 {
    // 读取 /proc/stat 获取CPU时间
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        if let Some(cpu_line) = content.lines().next() {
            if cpu_line.starts_with("cpu ") {
                let parts: Vec<&str> = cpu_line.split_whitespace().collect();
                if parts.len() >= 5 {
                    // user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle: u64 = parts[4].parse().unwrap_or(0);
                    let total = user + system + idle;

                    if total > 0 {
                        return ((user + system) as f64 / total as f64) * 100.0;
                    }
                }
            }
        }
    }
    0.0
}

/// 获取网络流量（简化实现）
fn get_network_stats() -> NetworkStats {
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        // 跳过前两行（标题行）
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 2 {
            // 累加所有接口的流量（排除 lo 回环接口）
            let mut rx_bytes = 0u64;
            let mut tx_bytes = 0u64;

            for line in lines.iter().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 17 {
                    let interface = parts[0].trim_end_matches(':');
                    if interface != "lo" { // 排除回环接口
                        if let Ok(rx) = parts[1].parse::<u64>() {
                            rx_bytes += rx;
                        }
                        if let Ok(tx) = parts[9].parse::<u64>() {
                            tx_bytes += tx;
                        }
                    }
                }
            }

            return NetworkStats {
                upload_bytes: tx_bytes,
                download_bytes: rx_bytes,
            };
        }
    }

    NetworkStats {
        upload_bytes: 0,
        download_bytes: 0,
    }
}
