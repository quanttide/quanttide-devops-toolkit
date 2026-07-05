# TODO — v0.2.3

> 分解 ROADMAP 为可执行的开发步骤。按依赖顺序排列。

## source::changelog 扩充

### Step 1: 扩展现有错误类型

- [ ] `ChangelogError` 新增 `Git(String)` 变体（git 命令失败）
- [ ] `ChangelogError` 新增 `Write(String)` 变体（文件写入失败）
- [ ] 完善 `Display` 实现覆盖新变体
- [ ] 单元测试：`test_changelog_error_display_git`
- [ ] 单元测试：`test_changelog_error_display_write`

### Step 2: 新增 `collect_git_log`

- [ ] `pub fn collect_git_log(repo_path, from_tag: Option<&str>) -> Result<String, ChangelogError>`
- [ ] `from_tag = Some(tag)` 时 range 为 `tag..HEAD`，`None` 时为 `HEAD`
- [ ] 错误处理：git 命令失败 → `ChangelogError::Git`
- [ ] 错误处理：空输出 → `ChangelogError::Git("没有新的提交记录")`
- [ ] 单元测试：有 tag 时只返回 tag 之后的提交
- [ ] 单元测试：无 tag 时返回全部提交
- [ ] 单元测试：空仓库返回错误
- [ ] 集成测试：真实 git 仓库（`tests/source_changelog.rs`）

### Step 3: 新增 `build_changelog_prompt`

- [ ] `pub fn build_changelog_prompt(git_log: &str, version: &str) -> String`
- [ ] 返回格式包含版本号
- [ ] 固定分类规则：Added / Changed / Fixed / Removed
- [ ] 固定要求：用中文、每类≤5条、合并概括
- [ ] 纯函数，不涉及任何 I/O
- [ ] 单元测试：输出包含 version 字符串
- [ ] 单元测试：输出包含 git_log 内容
- [ ] 单元测试：输出不包含未定义的关键词（如 "Deleted"）

### Step 4: 新增 `append_entry`

- [ ] `pub fn append_entry(path: &Path, version: &str, content: &str) -> Result<bool, ChangelogError>`
- [ ] 文件不存在时创建并写入头部 `# CHANGELOG\n`
- [ ] 版本已存在时跳过（返回 `Ok(false)`）
- [ ] 新条目插入到已有条目的最前面，放在已有第一个版本之前
- [ ] 使用 `jiff::Zoned::now()` 获取日期
- [ ] 版本号标准化（去掉 `v` 前缀、scope 前缀）
- [ ] 单元测试：创建新文件
- [ ] 单元测试：追加到已有文件
- [ ] 单元测试：版本已存在返回 false
- [ ] 单元测试：scope 版本号写入纯版本号
- [ ] 集成测试：完整读写 + 路径不存在

### Step 5: 更新示例

- [ ] `examples/changelog.rs` 改用 `collect_git_log` + `build_changelog_prompt`
- [ ] 删除手动 shell out 到 `git log` 的代码
- [ ] 验证 `cargo run --example changelog .` 正常运行

### Step 6: 测试覆盖率

- [ ] 全量测试通过
- [ ] 覆盖率 ≥ 95%
