use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

/// 性能监控器
pub struct PerformanceMonitor {
    start_time: Instant,
    operation_count: AtomicU64,
    total_duration: AtomicU64,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
        }
    }
    
    /// 记录操作
    pub fn record_operation<F, R>(&self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        
        result
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> PerformanceStats {
        let elapsed = self.start_time.elapsed();
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_duration = self.total_duration.load(Ordering::Relaxed);
        
        PerformanceStats {
            total_elapsed_ms: elapsed.as_millis() as u64,
            operation_count,
            total_operation_duration_ms: total_duration,
            avg_operation_duration_ms: if operation_count > 0 {
                total_duration as f64 / operation_count as f64
            } else {
                0.0
            },
            operations_per_second: if elapsed.as_secs() > 0 {
                operation_count as f64 / elapsed.as_secs() as f64
            } else {
                0.0
            },
        }
    }
}

/// 性能统计信息
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_elapsed_ms: u64,
    pub operation_count: u64,
    pub total_operation_duration_ms: u64,
    pub avg_operation_duration_ms: f64,
    pub operations_per_second: f64,
}

impl PerformanceStats {
    /// 获取性能评级
    pub fn get_performance_rating(&self) -> PerformanceRating {
        if self.avg_operation_duration_ms < 10.0 {
            PerformanceRating::Excellent
        } else if self.avg_operation_duration_ms < 100.0 {
            PerformanceRating::Good
        } else if self.avg_operation_duration_ms < 1000.0 {
            PerformanceRating::Fair
        } else {
            PerformanceRating::Poor
        }
    }
}

/// 性能评级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

/// 内存跟踪器
pub struct MemoryTracker {
    peak_memory: AtomicU64,
    current_memory: AtomicU64,
}

impl MemoryTracker {
    /// 创建新的内存跟踪器
    pub fn new() -> Self {
        Self {
            peak_memory: AtomicU64::new(0),
            current_memory: AtomicU64::new(0),
        }
    }
    
    /// 记录内存使用
    pub fn record_memory_usage(&self, bytes: u64) {
        self.current_memory.store(bytes, Ordering::Relaxed);
        let current = self.current_memory.load(Ordering::Relaxed);
        let peak = self.peak_memory.load(Ordering::Relaxed);
        
        if current > peak {
            self.peak_memory.store(current, Ordering::Relaxed);
        }
    }
    
    /// 获取内存统计信息
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_memory_bytes: self.current_memory.load(Ordering::Relaxed),
            peak_memory_bytes: self.peak_memory.load(Ordering::Relaxed),
        }
    }
}

/// 内存统计信息
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_memory_bytes: u64,
    pub peak_memory_bytes: u64,
}

impl MemoryStats {
    /// 格式化内存使用量
    pub fn format_memory_usage(&self) -> String {
        format!(
            "当前: {}, 峰值: {}",
            self.format_bytes(self.current_memory_bytes),
            self.format_bytes(self.peak_memory_bytes)
        )
    }
    
    /// 格式化字节数
    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
