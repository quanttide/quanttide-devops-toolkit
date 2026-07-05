# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。正序排列（近期版本在上）。

## [0.2.3] - 未发布

### 计划

- [ ] **`source::changelog` 扩充：CHANGELOG 生成逻辑标准化**
      
      背景：当前 `source::changelog` 只能解析/读取，不包含写入和 git log 收集能力。
      CLI 需要自己 shell out 到 git、构建 prompt、写入文件，逻辑散落。
      
      新增三个公共函数：
      
      ```rust
      /// 构建 LLM prompt（固定 CHANGELOG 生成规则）。纯函数，无依赖。
      pub fn build_changelog_prompt(git_log: &str, version: &str) -> String;
      
      /// 收集 git 提交记录（from_tag..HEAD），shell out 到 git。
      pub fn collect_git_log(repo_path: &Path, from_tag: Option<&str>)
          -> Result<String, ChangelogError>;
      
      /// 将新版本条目追加到 CHANGELOG 文件。文件不存在则初始化，版本已存在则跳过。
      pub fn append_entry(path: &Path, version: &str, content: &str)
          -> Result<bool, ChangelogError>;
      ```
      
      扩展现有 `ChangelogError` 以覆盖 Git 和写入错误。
      
      依赖：`build_changelog_prompt` 是纯函数（+0 依赖），`collect_git_log` shell out
      到 `git`（+0 依赖），`append_entry` 只用 `std::fs`（+0 依赖）。
      
      LLM 调用层在 CLI（qtcloud-devops）中完成，toolkit 不依赖 quanttide-agent。

- [ ] **示例更新：`examples/changelog.rs` 使用上述新 API**
      
      替换当前手动 shell out 到 git log 的实现，改用 `collect_git_log` 和
      `build_changelog_prompt`，演示完整调用链。
