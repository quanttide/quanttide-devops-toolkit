# quanttide-devops

QuantTide DevOps toolkit — 契约驱动的 DevOps 治理工具库。

## 安装

```toml
[dependencies]
quanttide-devops = "0.1"
```

## 快速开始

```rust
use std::path::Path;
use quanttide_devops::contract::{Contract, validate_version};

// 加载契约
let c = Contract::load(Path::new(".")).unwrap();

// 检查版本号格式
assert!(validate_version("v1.2.3"));

// 遍历 scope
for scope in &c.scopes {
    println!("{}: {}", scope.name, scope.dir);
}
```

## 架构

四维契约模型：

| 维度 | 模块 | 说明 |
|------|------|------|
| **Stage**（时序） | `stage.rs` | 生命周期阶段配置（build/test/release） |
| **Platform**（载体） | `platform.rs` | 外部治理载体（GitHub / CI / 制品库） |
| **Source**（事实源） | `source.rs` | 真相中心（版本号来源） |
| **Scope**（上下文） | `scope.rs` | 规则边界（语言/工具/阈值覆盖） |

## 功能

- 契约加载与解析（`.quanttide/devops/contract.yaml`）
- 版本号校验与标准化
- 多配置文件版本提取（Cargo.toml / pyproject.toml / package.json / pubspec.yaml）
- 语言与构建工具自动检测
- Scope 查找与便捷访问器

## 许可

Apache-2.0
