# `quanttide-devops`

QuantTide DevOps toolkit — 契约驱动的 DevOps 治理工具库。

## 安装

```toml
[dependencies]
quanttide-devops = "0.1"

# 可选：clap 集成（用于 CLI 工具）
# quanttide-devops = { version = "0.1", features = ["clap"] }
```

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
use quanttide_devops::source::version::{latest_tag, version_status};
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

## 许可

Apache-2.0
