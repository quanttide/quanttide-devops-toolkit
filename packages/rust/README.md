# quanttide-devops

QuantTide DevOps toolkit — 契约驱动的 DevOps 治理工具库。

## 安装

```toml
[dependencies]
quanttide-devops = "0.1"

# 可选：clap 集成（用于 CLI 工具）
# quanttide-devops = { version = "0.1", features = ["clap"] }
```

## 架构

四维契约模型 + 事实源模块：

| 维度 / 模块 | 位置 | 说明 |
|------|------|------|
| **Stage**（时序） | `contract::stage` | 生命周期阶段配置（build/test/release） |
| **Platform**（载体） | `contract::platform` | 外部治理载体（GitHub / CI / 制品库） |
| **Source**（事实源） | `contract::source` | 真相中心（版本号来源检测） |
| **Scope**（上下文） | `contract::scope` | 规则边界（语言/工具/阈值覆盖） |
| **CHANGELOG** | `source::changelog` | CHANGELOG.md 解析、release notes 提取 |
| **Git tag** | `source::git` | Git tag 读取、scope 过滤、版本一致性检查 |
| **ROADMAP** | `source::roadmap` | ROADMAP.md 解析、进度统计、格式验证 |

## 快速开始

### 契约加载

```rust
use std::path::Path;
use quanttide_devops::contract::Contract;

// 加载 `.quanttide/devops/contract.yaml`
let c = Contract::load(Path::new(".")).unwrap();

// 遍历 scope
for scope in &c.scopes {
    println!("{}: {} ({:?})", scope.name, scope.dir, scope.language);
}
```

### 版本号工具

```rust
use quanttide_devops::contract::{
    validate_version, normalize_version, read_all_config_versions,
};

assert!(validate_version("v1.2.3"));
assert!(validate_version("cli/v0.5.0-rc.1"));
assert!(!validate_version("1.2.3")); // 缺 v 前缀

assert_eq!(normalize_version("cli/v0.5.0"), "0.5.0");

let versions = read_all_config_versions(Path::new("."));
for (file, version) in &versions {
    println!("{}: {:?}", file, version);
}
```

### Git tag 版本检查

```rust
use std::path::Path;
use quanttide_devops::source::git::{latest_tag, version_status};
use quanttide_devops::contract::Scope;

// 获取 scope 最新 tag
let tag = latest_tag(Path::new("."), "cli").unwrap();
println!("latest: {:?}", tag);

// 版本一致性检查
let scope = Scope {
    name: "cli".into(),
    dir: "src/cli".into(),
    ..Default::default()
};
let status = version_status(Path::new("."), &scope).unwrap();
println!("consistent: {}", status.consistent);
```

### CHANGELOG 解析

```rust
use std::path::Path;
use quanttide_devops::source::changelog::Changelog;

let cl = Changelog::from_path(Path::new("CHANGELOG.md")).unwrap();
println!("latest: {:?}", cl.latest_version());
println!("notes: {:?}", cl.release_notes("0.1.5"));
```

### ROADMAP 进度统计

```rust
use std::path::Path;
use quanttide_devops::source::roadmap::Roadmap;

let rm = Roadmap::from_path(Path::new("ROADMAP.md")).unwrap();
// 查看每个版本的进度
for v in rm.versions() {
    println!("{} ({}): {}/{}", v.version, v.status, v.done, v.total);
}
println!("total: {}/{}", rm.total_done(), rm.total_all());

// 格式验证
let issues = rm.validate("rust");
for issue in &issues {
    println!("line {}: {}", issue.line, issue.message);
}
```

## 功能一览

| 功能 | 模块 | 版本 |
|------|------|------|
| 契约加载与解析（YAML） | `contract` | 0.1.0 |
| 枚举类型（SourceControl / Pipeline / Registry / Language / BuildTool） | `contract` | 0.1.0 |
| 语言与构建工具自动检测 | `contract::core` | 0.1.0 |
| Scope 查找与便捷访问器 | `contract::core` | 0.1.0 |
| 版本号校验与标准化 | `contract::version` | 0.1.2 |
| 多配置文件版本提取 | `contract::version` | 0.1.2 |
| Git tag 读取、scope 过滤、版本一致性 | `source::git` | 0.1.3 |
| CHANGELOG 解析、release notes 提取 | `source::changelog` | 0.1.4 |
| ROADMAP 解析、进度统计、格式验证 | `source::roadmap` | 0.1.5 |
| 可选 clap feature（CLI 集成） | `contract`（feature） | 0.1.2 |

## 许可

Apache-2.0
