#[test]
fn test_scope_deserialize_wrong_type() {
    let result = quanttide_devops::contract::load_from_str("scopes: not_a_map\n");
    assert!(result.is_err());
}

#[test]
fn test_scope_deserialize_type_error() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        r#"
scopes:
  cli:
    dir:
      nested: value
"#,
    )
    .unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("解析失败") || err.to_string().contains("作用域"));
}

#[test]
fn test_scope_deserialize_missing_dir() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join(".quanttide/devops");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("contract.yaml"),
        r#"
scopes:
  cli:
    language: rust
"#,
    )
    .unwrap();
    let err = quanttide_devops::contract::load(d.path()).unwrap_err();
    assert!(err.to_string().contains("dir"));
}
