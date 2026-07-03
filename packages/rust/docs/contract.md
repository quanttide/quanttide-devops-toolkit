# contract/ — 四维契约模型

> 四维架构：Stages（时序）、Platforms（载体）、Sources（事实源）、Scopes（边界）。
> 对应文件 `.quanttide/devops/contract.yaml`，由 `load()` 解析加载。

## 定位

`contract/` 是 toolkit 的核心模块，定义 DevOps 治理的契约模型。
与 `source/`（事实源实现）平级，`contract/` 负责"定义"，`source/` 负责"读取"。

| 模块 | 职责 |
|------|------|
| `contract/core.rs` | Contract 结构体 + 便捷访问器 + `detect_language_by_files` |
| `contract/stage.rs` | Stage（时序维度）：build / test / release 阶段配置 |
| `contract/platform.rs` | Platform（载体维度）：source_control / pipeline / artifact_registry |
| `contract/source.rs` | Source（事实源维度）：version source_type / path |
| `contract/scope.rs` | Scope（上下文维度）：name / dir / language / build_tool / 覆盖 |
| `contract/version.rs` | 版本号工具：validate / normalize / extract |
| `contract/error.rs` | ContractError 类型 |
| `contract/mod.rs` | 模块出口 + `load()` / `load_from_str()` |

## 四维模型

```yaml
stages:       # 时序 — 生命周期各阶段配置
  build:      #   构建阶段（command: Option<String>）
  test:       #   测试阶段（command + threshold: f64, default 70.0）
  release:    #   发布阶段（changelog + pre_publish scripts）

platform:     # 载体 — 外部治理平台
  source_control:  # github / gitlab / gitee
  pipeline:        # github_actions / gitlab_ci / jenkins
  artifact_registry: # crates / pypi / pubdev / npm / docker / none

sources:      # 事实源 — 版本号从哪读
  version:    #   type: cargo / pyproject / tag / pubspec / package.json / auto
              #   path: 自定义路径（可选）

scopes:       # 上下文 — 规则边界，继承全局默认值
  <name>:
    dir:           # 子目录路径（必填）
    language:      # rust / python / go / dart / typescript（auto 检测）
    build_tool:    # cargo / uv / go / flutter / npm（auto 检测）
    registry:      # 覆盖全局 platform.artifact_registry
    release:       # 覆盖全局 stages.release（changelog / pre_publish）
    test_threshold:# 覆盖全局 stages.test.threshold
    ci_workflow:   # 自定义 CI workflow 名称
```

## 依赖

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
```

## 核心 API

### 加载

```rust
/// 从 .quanttide/devops/contract.yaml 加载契约。
pub fn load(repo_path: &Path) -> Result<Contract, ContractError>

/// 从 YAML 字符串解析契约（纯函数，不依赖文件系统）。
pub fn load_from_str(s: &str) -> Result<Contract, ContractError>
```

### 便捷访问

```rust
impl Contract {
    /// Scope 级发布配置（scope 优先 → 全局默认）。
    pub fn scope_release(&self, scope: &Scope) -> &StageRelease

    /// Scope 级测试阈值（scope 优先 → 全局 70.0）。
    pub fn scope_test_threshold(&self, scope: &Scope) -> f64

    /// 按路径匹配 scope（最长前缀匹配）。
    pub fn find_scope_by_path(&self, current_dir: &Path) -> Option<&Scope>

    /// 验算：检查所有 scope.dir 是否存在。
    pub fn validate(&self, repo_path: &Path) -> Vec<String>
}
```

### 语言/工具检测

```rust
/// 按标志文件检测编程语言。
pub fn detect_language_by_files(dir: &Path) -> Language
```

### 版本号工具

```rust
pub fn validate_version(version: &str) -> bool
pub fn normalize_version(version: &str) -> String
pub fn read_all_config_versions(dir: &Path) -> Vec<(String, Option<String>)>
```

### 枚举类型

| 类型 | 变体 | 说明 |
|------|------|------|
| `Language` | Rust / Python / Go / Dart / TypeScript / Unknown | 编程语言 |
| `BuildTool` | Cargo / Uv / Go / Flutter / Npm / Unknown | 构建工具 |
| `SourceType` | Cargo / Pyproject / TagOnly / Pubspec / PackageJson / Auto | 版本号来源 |
| `SourceControl` | Github / Gitlab / Gitee | 代码托管 |
| `Pipeline` | GithubActions / GitlabCi / Jenkins | CI 平台 |
| `Registry` | Crates / PyPI / PubDev / Npm / GitHubReleases / Docker / None | 制品库 |

### 错误类型

```rust
pub enum ContractError {
    Io(io::Error),         // 文件读取失败
    Parse(String),         // YAML 解析失败
    NotFound,              // 文件不存在
}
```

## 覆盖语义（浅覆盖）

Scope 级有值就用 scope 的，没有就用全局的。不是深度合并。

```yaml
stages:
  test:
    threshold: 70          # 全局默认

scopes:
  cli:
    dir: src/cli
    # test_threshold 未设 → 使用全局 70
  sensitive:
    dir: src/sensitive
    test_threshold: 95     # 覆盖全局
```

## 折叠语义（JSON/YAML 映射 → Vec<Scope>）

`contract.yaml` 中 `scopes` 是映射格式：

```yaml
scopes:
  cli:
    dir: src/cli
```

解析时折叠为 `Vec<Scope>`，其中 `name` 从映射键取值。
使用自定义 `deserialize_scopes` visitor 实现。

## 版本号提取

`read_all_config_versions` 支持 4 种配置文件格式：

| 文件 | 提取方式 | 示例 |
|------|---------|------|
| `Cargo.toml` | `version = "..."` 键值对 | `version = "0.1.0"` |
| `pyproject.toml` | `version = "..."` 键值对 | `version = "0.1.0"` |
| `package.json` | `"version": "..."` JSON | `"version": "0.1.0"` |
| `pubspec.yaml` | `version: ...` YAML | `version: 0.1.0` |

## 测试策略

- **单元测试**（`#[cfg(test)]` 在各自模块中）：纯函数 + 构造 YAML 字符串验证解析
- **集成测试**（`tests/integration.rs`）：真实文件系统覆盖 `load()`、`detect_language_by_files`、`read_all_config_versions` 的 I/O 路径
- **覆盖率目标**：> 95%（当前 98.39%）

## 安全/默认值

- 所有枚举有 `Unknown` 变体兜底，未知输入不崩溃
- `Contract::default()` 可安全使用（空 scopes，标准配置）
- 旧格式 `scopes: { cli: src/cli }` 不再支持，所有 scope 必须使用对象格式
