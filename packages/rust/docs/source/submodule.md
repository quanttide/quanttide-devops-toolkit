# Submodule 重构蓝图

> 现状：`examples/submodule.rs` 是一个 723 行的早期原型，混合 `git2` 和 CLI 命令，错误处理用 `Box<dyn Error>`。
> 目标：渐进式重构，先是干净的 example，再视情况演化为库模块。

## 阶段划分

```
Phase 1 ─ 重写 example
  当前 → 干净的 examples/submodule.rs
  只改这一个文件，不动库代码

Phase 2 ─ 沉淀通用能力
  从 example 中提取库空缺的工具函数
  补充到 source/git.rs 或新增 source/git_utils.rs

Phase 3 ─ 评估是否入库
  若 submodule 能力被 CLI 或其他模块依赖，再考虑
  作为 source/submodule/ 移入库中
```

当前处于 **Phase 1**。

## Phase 1：example 重写方向

### 保留

- 全部 7 种 SubmoduleStatus（Dirty / Orphaned / Detached / Uninitialized / BehindRemote / AheadOfParent / Clean）
- 全部扫描逻辑（RepoState::scan / scan_offline / scan_all）
- 全部编辑操作（GitSubmoduleEditor::sync_to_parent / sync_all_to_parent / status）
- 全部 60+ 个测试（已验证的边界覆盖）
- 中文错误消息（面向国内团队）

### 改进

| 问题 | 现状 | 改后 |
|------|------|------|
| 错误类型 | `Box<dyn Error>` | 局部 `Error` 枚举 |
| 远程状态返回 | 7 元组 `(…, …, bool, usize, usize, bool, bool)` | 命名结构体 |
| 状态判定参数 | 9 个扁平的 bool/scalar | 输入结构体 |
| 测试辅助命名 | `h()` / `dh()` | 自解释名称或直接构造 |
| 库空缺标记 | 无 | 文件头 + 行内 `// gap:` 注释 |
| 库类型使用 | 自包含，不用 `quanttide_devops` | 最大限度使用库已有类型和能力 |
| git 操作方式 | 混合 `git2` + `std::process::Command` | 最大限度使用 `git2` |

### 不改

- ❌ 不引入 `source/submodule/` 目录进库
- ❌ 不和 `contract/` 模型做任何关联

### 库空缺标记

重写 example 时，凡是需要但库尚未提供的能力，用 `// gap:` 注释标记：

```rust
// gap: quanttide_devops 缺少通用 revwalk 工具函数
// gap: parse_oid / count_between_opt 应提为 source/git_utils 模块
```

同时更新 `Phase 2` 的沉淀清单，作为补充库能力的待办项。


## 不做的（长期不变）

- 不做 `git submodule add / deinit / update --init` 等管理操作
- 不做 `git submodule foreach` 的封装
- 不与 `Contract` 模型强绑定
- 不引入额外的第三方依赖
