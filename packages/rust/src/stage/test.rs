/// 测试结果汇总。
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TestSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

/// 覆盖率数据。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Coverage {
    pub percentage: f64,
    pub threshold: f64,
}

impl Coverage {
    pub fn met(&self) -> bool {
        self.percentage >= self.threshold
    }
}

/// 质量审计结果。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AuditReport {
    pub total_tests: usize,
    pub total_pub_fns: usize,
    pub pure_pub_fns: usize,
    pub tested_pub_fns: usize,
    pub error_variants: usize,
    pub tested_variants: usize,
    pub uncovered_fns: Vec<(String, String)>,
    pub uncovered_variants: Vec<String>,
    pub coverage_pct: f64,
    pub coverage_threshold: f64,
    pub gates_met: bool,
}

/// I/O 函数模式：这些名称的函数不要求单元测试（集成测试覆盖即可）。
///
/// 注意：纯函数（parse_/determine_/build_/fmt_/map_/normalize_/apply_/extract_/is_）
/// 不应在此列表。
const IO_FN_PATTERNS: &[&str] = &[
    "status", "status_to", "run", "run_direct", "run_scoped",
    "sync", "sync_all", "sync_to_parent", "sync_all_to_parent",
    "publish", "scan", "scan_offline",
    "push_submodule", "push_parent", "fetch_submodule", "rebase_submodule",
    "check_command", "check_syntax", "check_ci", "check_deps",
    "clear_cache", "save_test_summary",
    "ensure_", "delete_", "create_",
    "git_output", "llm_decide", "llm_changelog", "edit_llm",
    "detect_version", "detect_single_scope", "detect_project_type",
    "resolve_roadmap_path",
    "print_status", "print_status_to", "print_scope_audit",
    "collect_git_log", "collect_tags_with_scope", "collect_test_summary_from_run",
    "load_contract_scopes", "load_scopes_map",
    "apply_rule_fixes",
    "collect_rs_files", "collect_test_fns", "collect_pub_fns", "collect_error_variants",
];

/// 判断函数名是否为 I/O 型（无需单元测试，集成测试覆盖即可）。
pub fn is_io_fn(name: &str) -> bool {
    IO_FN_PATTERNS.iter().any(|p| {
        if p.ends_with('_') {
            name.starts_with(p)
        } else {
            name == *p
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TestSummary ──────────────────────────────────────────────

    #[test]
    fn test_test_summary_default() {
        let s = TestSummary::default();
        assert_eq!(s.total, 0);
    }

    #[test]
    fn test_test_summary_serde_roundtrip() {
        let s = TestSummary { total: 10, passed: 8, failed: 1, skipped: 1 };
        let json = serde_json::to_string(&s).unwrap();
        let back: TestSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn test_test_summary_serde_all_zero() {
        let s = TestSummary { total: 0, passed: 0, failed: 0, skipped: 0 };
        let json = serde_json::to_string(&s).unwrap();
        let back: TestSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    // ── Coverage ─────────────────────────────────────────────────

    #[test]
    fn test_coverage_met() {
        let c = Coverage { percentage: 90.0, threshold: 80.0 };
        assert!(c.met());
    }

    #[test]
    fn test_coverage_not_met() {
        let c = Coverage { percentage: 70.0, threshold: 80.0 };
        assert!(!c.met());
    }

    #[test]
    fn test_coverage_met_exact() {
        let c = Coverage { percentage: 80.0, threshold: 80.0 };
        assert!(c.met());
    }

    #[test]
    fn test_coverage_met_zero_threshold() {
        let c = Coverage { percentage: 0.0, threshold: 0.0 };
        assert!(c.met());
    }

    #[test]
    fn test_coverage_default() {
        let c = Coverage::default();
        assert_eq!(c.percentage, 0.0);
        assert_eq!(c.threshold, 0.0);
    }

    // ── AuditReport ──────────────────────────────────────────────

    #[test]
    fn test_audit_report_default() {
        let r = AuditReport::default();
        assert_eq!(r.total_tests, 0);
        assert!(!r.gates_met);
    }

    #[test]
    fn test_audit_report_gates_met() {
        let r = AuditReport {
            total_tests: 10,
            total_pub_fns: 5,
            pure_pub_fns: 5,
            tested_pub_fns: 3,
            error_variants: 2,
            tested_variants: 2,
            uncovered_fns: vec![],
            uncovered_variants: vec![],
            coverage_pct: 100.0,
            coverage_threshold: 80.0,
            gates_met: true,
        };
        assert!(r.gates_met);
    }

    // ── is_io_fn ──────────────────────────────────────────────────

    #[test]
    fn test_is_io_fn_status() {
        assert!(is_io_fn("status"));
        assert!(is_io_fn("publish"));
        assert!(is_io_fn("clear_cache"));
    }

    #[test]
    fn test_is_io_fn_prefix() {
        assert!(is_io_fn("ensure_changelog"));
        assert!(is_io_fn("delete_tag"));
        assert!(is_io_fn("create_release"));
    }

    #[test]
    fn test_is_io_fn_pure() {
        assert!(!is_io_fn("parse_version"));
        assert!(!is_io_fn("normalize_version"));
        assert!(!is_io_fn("build_version"));
    }
}
