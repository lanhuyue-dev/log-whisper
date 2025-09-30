#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
大文件测试工具 - 生成测试日志文件并验证应用性能
"""

import os
import time
import random
from datetime import datetime, timedelta

def generate_large_log_file(filename, size_mb=100, lines_per_mb=1000):
    """生成大日志文件用于测试"""
    
    print(f"🚀 生成测试日志文件: {filename} ({size_mb}MB)")
    
    # 预定义的日志模板
    log_templates = [
        "{timestamp} INFO [main] Application started successfully",
        "{timestamp} DEBUG [worker-{worker_id}] Processing request {request_id}",
        "{timestamp} ERROR [database] Connection failed: {error_msg}",
        "{timestamp} WARN [cache] Cache miss for key: {cache_key}",
        "{timestamp} INFO [api] User {user_id} logged in from {ip_address}",
        "{timestamp} ERROR [payment] Payment failed for order {order_id}: {payment_error}",
        "{timestamp} DEBUG [scheduler] Job {job_id} completed in {duration}ms",
        "{timestamp} INFO [audit] Security event: {security_event}",
        "{timestamp} WARN [memory] Memory usage: {memory_usage}MB",
        "{timestamp} ERROR [network] Network timeout: {network_error}",
    ]
    
    # 错误消息池
    error_messages = [
        "Connection timeout after 30 seconds",
        "Invalid credentials provided",
        "Resource not found",
        "Internal server error",
        "Database deadlock detected",
        "Out of memory exception",
        "Network unreachable",
        "Permission denied",
    ]
    
    # 生成随机数据
    base_time = datetime.now() - timedelta(days=1)
    total_lines = size_mb * lines_per_mb
    
    with open(filename, 'w', encoding='utf-8') as f:
        for i in range(total_lines):
            # 选择随机模板
            template = random.choice(log_templates)
            
            # 生成时间戳
            log_time = base_time + timedelta(seconds=i * 0.1)
            timestamp = log_time.strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
            
            # 填充模板变量
            log_line = template.format(
                timestamp=timestamp,
                worker_id=random.randint(1, 10),
                request_id=f"req_{random.randint(100000, 999999)}",
                error_msg=random.choice(error_messages),
                cache_key=f"cache_{random.randint(1000, 9999)}",
                user_id=random.randint(1001, 9999),
                ip_address=f"192.168.{random.randint(1, 255)}.{random.randint(1, 255)}",
                order_id=f"order_{random.randint(100000, 999999)}",
                payment_error=random.choice(error_messages),
                job_id=f"job_{random.randint(1000, 9999)}",
                duration=random.randint(10, 5000),
                security_event=f"login_attempt_{random.choice(['success', 'failed'])}",
                memory_usage=random.randint(100, 2048),
                network_error=random.choice(error_messages)
            )
            
            f.write(log_line + '\n')
            
            # 显示进度
            if i % 10000 == 0 and i > 0:
                progress = (i / total_lines) * 100
                print(f"  进度: {progress:.1f}% ({i:,}/{total_lines:,} 行)")
    
    # 验证文件大小
    actual_size = os.path.getsize(filename)
    actual_size_mb = actual_size / (1024 * 1024)
    
    print(f"✅ 文件生成完成:")
    print(f"  - 文件名: {filename}")
    print(f"  - 实际大小: {actual_size_mb:.2f}MB")
    print(f"  - 总行数: {total_lines:,}")
    print(f"  - 平均行长: {actual_size // total_lines} 字节")

def create_test_files():
    """创建各种大小的测试文件"""
    
    test_files = [
        ("small_test.log", 1, 1000),      # 1MB
        ("medium_test.log", 10, 1000),    # 10MB  
        ("large_test.log", 100, 1000),    # 100MB
        ("huge_test.log", 500, 1000),     # 500MB
        ("extreme_test.log", 1000, 1000), # 1GB
    ]
    
    print("🔧 创建测试文件集合...")
    
    for filename, size_mb, lines_per_mb in test_files:
        if not os.path.exists(filename):
            try:
                generate_large_log_file(filename, size_mb, lines_per_mb)
                print(f"  ✅ {filename} 创建成功")
            except Exception as e:
                print(f"  ❌ {filename} 创建失败: {e}")
        else:
            print(f"  ⏭️ {filename} 已存在，跳过")
    
    print("\n📊 测试文件清单:")
    for filename, size_mb, _ in test_files:
        if os.path.exists(filename):
            actual_size = os.path.getsize(filename) / (1024 * 1024)
            print(f"  - {filename}: {actual_size:.2f}MB")

def performance_test_report():
    """生成性能测试报告"""
    
    print("\n📋 大文件支持性能测试指南")
    print("=" * 50)
    
    print("\n🎯 测试目标:")
    print("1. 验证内存映射文件读取")
    print("2. 测试虚拟滚动性能")
    print("3. 验证分块加载机制")
    print("4. 测试内存使用控制")
    
    print("\n🧪 测试步骤:")
    print("1. 启动 LogWhisper 应用")
    print("2. 依次加载测试文件:")
    print("   - small_test.log (1MB) - 基础功能测试")
    print("   - medium_test.log (10MB) - 中等文件测试")
    print("   - large_test.log (100MB) - 大文件测试")
    print("   - huge_test.log (500MB) - 超大文件测试")
    print("   - extreme_test.log (1GB) - 极限测试")
    
    print("\n📊 性能指标:")
    print("- 文件加载时间 (< 10秒)")
    print("- 内存使用量 (< 500MB)")
    print("- 滚动响应时间 (< 100ms)")
    print("- 搜索响应时间 (< 1秒)")
    
    print("\n⚠️ 预期改进:")
    print("- 不再出现 'out of memory' 错误")
    print("- 大文件加载时间显著减少")
    print("- 内存使用量保持稳定")
    print("- 滚动和交互保持流畅")
    
    print("\n🐛 如果出现问题:")
    print("1. 检查浏览器控制台错误")
    print("2. 查看应用日志文件")
    print("3. 监控系统内存使用")
    print("4. 验证虚拟滚动是否启用")

def main():
    """主函数"""
    print("🔍 LogWhisper 大文件支持测试工具")
    print("=" * 40)
    
    try:
        # 创建测试文件
        create_test_files()
        
        # 显示测试指南
        performance_test_report()
        
        print("\n🎉 测试文件准备完成！")
        print("请使用生成的测试文件验证 LogWhisper 的大文件处理能力。")
        
    except Exception as e:
        print(f"\n❌ 测试工具执行失败: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())