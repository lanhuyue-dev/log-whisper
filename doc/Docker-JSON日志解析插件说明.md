# Docker JSON日志解析插件

## 概述

为log-whisper项目新增了Docker容器JSON日志解析插件，支持解析Docker容器标准JSON日志格式，并提供友好的可视化展示。

## 插件功能

### 核心特性

1. **JSON格式检测**: 自动识别Docker容器JSON日志格式
2. **结构化解析**: 解析log、stream、time等字段
3. **流类型识别**: 区分stdout和stderr流
4. **错误检测**: 自动识别错误和警告信息
5. **时间格式化**: 支持RFC3339时间戳解析
6. **多块渲染**: 为复杂日志提供原始JSON和格式化内容两种视图

### 支持的日志格式

Docker容器标准JSON日志格式：
```json
{
  "log": "日志内容\n",
  "stream": "stdout|stderr", 
  "time": "RFC3339时间戳"
}
```

### 示例

#### 输入日志
```json
{"log":"Starting application server...\n","stream":"stdout","time":"2024-09-30T08:00:00.123456789Z"}
{"log":"Error: Failed to connect to Redis\n","stream":"stderr","time":"2024-09-30T08:00:07.890123456Z"}
```

#### 解析结果
- **stdout日志**: 显示为INFO类型，使用📤图标
- **stderr日志**: 显示为ERROR类型，使用❌图标
- **时间戳**: 格式化为可读的UTC时间
- **内容**: 自动去除换行符和多余空白

## 技术实现

### 文件结构
```
src-tauri/src/plugins/
├── docker_json.rs          # Docker JSON解析插件实现
├── mod.rs                  # 模块导出（已更新）
└── registry.rs             # 插件注册中心（已注册新插件）
```

### 核心组件

#### DockerJsonRenderer
- 实现`LogRenderer` trait
- 优先级: 15（介于MyBatis和JSON插件之间）
- 支持文件类型: `*.log`, `*.json`, `*-json.log`, `docker-*.log`
- 性能评级: High
- 内存使用: Medium

#### DockerLogRecord结构体
```rust
pub struct DockerLogRecord {
    pub log: String,        // 日志内容
    pub stream: String,     // 流类型 (stdout, stderr)
    pub time: String,       // 时间戳
}
```

### 解析逻辑

1. **格式检测**:
   - 检查JSON开始符号`{`
   - 验证必需字段`log`、`stream`、`time`存在

2. **内容解析**:
   - 使用serde_json解析JSON结构
   - 提取各字段内容
   - 处理转义字符

3. **分类逻辑**:
   - stderr流: 自动标记为ERROR
   - stdout流: 根据内容检测错误关键词
   - 支持的错误关键词: "error", "exception", "warn"

4. **块生成**:
   - 格式化内容块: 包含时间戳、流类型、格式化内容
   - JSON原始块: 用于复杂或长内容的详细查看

## 测试验证

### 测试用例

创建了`data/sample-docker-json.log`测试文件，包含：
- ✅ 正常stdout日志
- ✅ stderr错误日志  
- ✅ 多行错误信息
- ✅ 应用启动/停止日志
- ✅ 系统监控信息
- ✅ 异常堆栈信息

### 验证结果

通过独立测试程序验证，所有测试用例均正确解析：
- 正确识别14行Docker JSON日志
- 准确区分stdout/stderr流类型
- 正确解析时间戳格式
- 成功提取和格式化日志内容
- 准确识别错误和警告信息

## 配置选项

插件支持以下配置：
```json
{
  "enabled": true,
  "name": "DockerJSON",
  "description": "Docker容器JSON日志解析器", 
  "priority": 15,
  "supported_formats": ["docker_json"],
  "features": {
    "parse_timestamps": true,
    "stream_detection": true, 
    "error_highlighting": true,
    "json_formatting": true
  }
}
```

## 使用方法

1. **自动检测**: 插件会自动检测Docker JSON格式的日志文件
2. **手动选择**: 可在插件列表中选择"DockerJSON"插件
3. **文件类型**: 支持`.log`、`.json`、`*-json.log`等扩展名

## 性能特点

- **高性能**: 使用高效的JSON解析算法
- **低内存**: 增量解析，避免大量内存占用
- **快速识别**: 基于字符串匹配的快速格式检测
- **流式处理**: 支持大文件逐行处理

## 未来扩展

计划支持的功能：
1. **多容器日志**: 支持解析容器ID和名称
2. **日志聚合**: 按时间线合并多个容器日志
3. **过滤功能**: 按流类型、时间范围过滤
4. **导出功能**: 导出解析后的结构化数据
5. **实时监控**: 支持实时Docker日志流

## 依赖项

新增依赖：
- `uuid`: 用于生成唯一的渲染块ID
- `serde_json`: JSON序列化和反序列化
- `chrono`: 时间戳处理

## 贡献

插件遵循log-whisper项目的插件化架构设计，完全兼容现有系统，可作为其他类似插件的开发参考。