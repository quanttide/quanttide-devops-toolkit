use quanttide_devops::contract::Registry;

#[test]
fn test_registry_serde_roundtrip() {
    let cases = vec![
        (Registry::Crates, "crates"),
        (Registry::PyPI, "pypi"),
        (Registry::PubDev, "pubdev"),
        (Registry::Npm, "npm"),
        (Registry::GitHubReleases, "github_releases"),
        (Registry::Docker, "docker"),
        (Registry::None, "none"),
    ];
    for (reg, yaml) in cases {
        let serialized = serde_yaml::to_string(&reg).unwrap();
        let trimmed = serialized.trim();
        assert_eq!(trimmed, yaml);
        let deserialized: Registry = serde_yaml::from_str(trimmed).unwrap();
        assert_eq!(deserialized, reg);
    }
}
