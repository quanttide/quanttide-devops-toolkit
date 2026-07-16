/// CI 运行记录。
#[derive(Debug, PartialEq)]
pub struct CiRun {
    pub conclusion: String,
    pub title: String,
    pub branch: String,
    pub number: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_run_construction() {
        let run = CiRun {
            conclusion: "success".into(),
            title: "ci".into(),
            branch: "main".into(),
            number: "42".into(),
        };
        assert_eq!(run.conclusion, "success");
        assert_eq!(run.number, "42");
    }

    #[test]
    fn test_ci_run_debug() {
        let run = CiRun {
            conclusion: "failure".into(),
            title: "build".into(),
            branch: "feat/x".into(),
            number: "7".into(),
        };
        assert!(format!("{:?}", run).contains("failure"));
    }
}
