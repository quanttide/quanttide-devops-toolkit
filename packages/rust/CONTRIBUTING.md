# CONTRIBUTING

## 命名规则

- 包名（`[package].name`）即 lib 名，**不设 `[lib] name` 覆盖**
- 引用时用包名的 snake_case 形式：`quanttide-devops` → `quanttide_devops`
- 所有 crate 统一此规则，不留别名

## 文档

- 所有 `pub` 类型、字段、方法必须写 `///` 文档注释
- 核心类型配文档测试（`///` 中 `\`\`\`` 代码块），`cargo test` 自动运行
- 文档测试中通过 `use quanttide_devops::...` 引用本 crate
