//! Git submodule health scanner and sync tool.
//!
//! Scans a repository's submodules, detects their status (dirty, detached,
//! behind remote, etc.), and syncs them back to a healthy state.
//!
//! Core git operations are abstracted behind the [`Git`] trait so the
//! submodule scanner and editor can be tested with a mock backend.
//! The real implementation [`RealGit`] uses `git2`.
//!
//! # Usage
//!
//! ```ignore
//! use git_submodule::{RepoState, GitSubmoduleEditor};
//!
//! let state = RepoState::scan(path)?;
//! println!("{}/{} submodules clean", state.clean_count, state.total);
//!
//! let editor = GitSubmoduleEditor::new(root);
//! editor.sync_to_parent("libs/shared")?;
//! ```

fn main() {
    println!("git_submodule — Git submodule health scanner and sync tool.");
    println!("Run `cargo test --example git_submodule` to run tests.");
}

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

// ── GitOps trait ───────────────────────────────────────────────────────
//
// Abstracts common git operations so the submodule editor doesn't depend
// on `git2` directly. Swap in a mock to test editor logic without real git.

/// Common git operations needed by the submodule editor.
pub trait Git {
    /// Return the current branch name for the repo at `path`, or `None` if detached.
    fn current_branch(path: &Path) -> Option<String>;

    /// Whether remote `origin` exists for the repo at `path`.
    fn has_origin(path: &Path) -> bool;

    /// Number of commits in `branch` not in `origin/<branch>`.
    fn ahead_count(path: &Path, branch: &str) -> usize;

    /// Fetch refs from remote `origin`.
    fn fetch(path: &Path);
}

/// Real git implementation backed by `git2`.
pub struct RealGit;

impl Git for RealGit {
    fn current_branch(path: &Path) -> Option<String> {
        repo(path)?.head().ok()?.shorthand().map(String::from)
    }

    fn has_origin(path: &Path) -> bool {
        repo(path).is_some_and(|r| r.find_remote("origin").is_ok())
    }

    fn ahead_count(path: &Path, branch: &str) -> usize {
        let Some(repo) = repo(path) else { return 0 };
        let local = repo.refname_to_id(&format!("refs/heads/{}", branch)).ok();
        let remote = repo
            .refname_to_id(&format!("refs/remotes/origin/{}", branch))
            .ok();
        count_between(&repo, remote, local)
    }

    fn fetch(path: &Path) {
        let Some(repo) = repo(path) else { return };
        let Ok(mut remote) = repo.find_remote("origin") else {
            return;
        };
        let mut opts = git2::FetchOptions::new();
        opts.download_tags(git2::AutotagOption::None);
        let mut cb = git2::RemoteCallbacks::new();
        cb.transfer_progress(|_| true);
        opts.remote_callbacks(cb);
        let _ = remote.fetch(
            &["+refs/heads/*:refs/remotes/origin/*"],
            Some(&mut opts),
            None,
        );
    }
}

fn revert_commit(path: &Path) {
    let Some(repo) = repo(path) else { return };
    let Ok(head) = repo.head() else { return };
    let Ok(commit) = head.peel_to_commit() else {
        return;
    };
    let Ok(parent) = commit.parent(0) else { return };
    let _ = repo.reset(&parent.as_object(), git2::ResetType::Hard, None);
}

fn repo(path: &Path) -> Option<git2::Repository> {
    git2::Repository::open(path).ok()
}

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    RepoOpen(String),
    Git2(git2::Error),
    Operation(String),
    NotFound(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoOpen(p) => write!(f, "无法打开仓库: {}", p),
            Self::Git2(e) => write!(f, "git2 错误: {}", e),
            Self::Operation(msg) => write!(f, "操作失败: {}", msg),
            Self::NotFound(name) => write!(f, "子模块未找到: {}", name),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Git2(e) => Some(e),
            _ => None,
        }
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Self::Git2(e)
    }
}

// ── CommitHash ─────────────────────────────────────────────────────────

/// A git object ID (OID) wrapper. Display truncates to 7 characters.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CommitHash(pub String);

impl std::fmt::Display for CommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[..self.0.len().min(7)])
    }
}

impl Default for CommitHash {
    fn default() -> Self {
        Self("0000000000000000000000000000000000000000".into())
    }
}

// ── SubmoduleStatus ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SubmoduleStatus {
    Dirty,
    Orphaned,
    Detached,
    Uninitialized,
    BehindRemote,
    AheadOfParent,
    Clean,
}

impl SubmoduleStatus {
    pub fn priority(self) -> u8 {
        match self {
            Self::Dirty => 0,
            Self::Orphaned => 1,
            Self::Detached => 2,
            Self::Uninitialized => 3,
            Self::BehindRemote => 4,
            Self::AheadOfParent => 5,
            Self::Clean => 6,
        }
    }
}

// ── RemoteState / StatusInput ──────────────────────────────────────────

struct RemoteState {
    local_head: CommitHash,
    remote_head: CommitHash,
    detached: bool,
    ahead: usize,
    behind: usize,
    orphaned: bool,
    remote_unreachable: bool,
}

struct StatusInput {
    uninitialized: bool,
    dirty: bool,
    detached: bool,
    orphaned: bool,
    remote_unreachable: bool,
    ahead: usize,
    behind: usize,
    local_head: CommitHash,
    parent_pointer: CommitHash,
}

// ── Submodule ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct Submodule {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub tracked_branch: String,
    pub parent_pointer: CommitHash,
    pub local_head: CommitHash,
    pub remote_head: CommitHash,
    pub status: SubmoduleStatus,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub remote_unreachable: bool,
}

// ── RepoState (scanning) ───────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoState {
    pub root_path: PathBuf,
    pub submodules: Vec<Submodule>,
    pub total: usize,
    pub clean_count: usize,
    pub needs_attention: Vec<String>,
}

impl RepoState {
    /// Scan all submodules (fetches remotes).
    pub fn scan(root: &Path) -> Result<Self, Error> {
        Self::scan_with_options(root, false)
    }

    /// Scan without fetching (offline).
    pub fn scan_offline(root: &Path) -> Result<Self, Error> {
        Self::scan_with_options(root, true)
    }

    /// Convenience: scan → (submodules, AggregateStatus).
    pub fn scan_all(root: &Path) -> Result<(Vec<Submodule>, AggregateStatus), Error> {
        let state = Self::scan(root)?;
        let agg = AggregateStatus::from_submodules(&state.submodules);
        Ok((state.submodules, agg))
    }

    fn scan_with_options(root: &Path, offline: bool) -> Result<Self, Error> {
        let repo = git2::Repository::open(root)
            .map_err(|e| Error::RepoOpen(format!("{}: {}", root.display(), e)))?;
        let gitmodules = root.join(".gitmodules");

        let submodules = if gitmodules.exists() {
            let mut git_submodules = repo.submodules()?;
            git_submodules.sort_by(|a, b| a.name().cmp(&b.name()));
            git_submodules
                .iter()
                .map(|sm| Self::scan_single(root, sm, &repo, offline))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        let total = submodules.len();
        let clean_count = submodules
            .iter()
            .filter(|s| s.status == SubmoduleStatus::Clean)
            .count();
        let needs_attention: Vec<String> = submodules
            .iter()
            .filter(|s| s.status != SubmoduleStatus::Clean)
            .map(|s| s.name.clone())
            .collect();

        Ok(RepoState {
            root_path: root.to_path_buf(),
            submodules,
            total,
            clean_count,
            needs_attention,
        })
    }

    fn scan_single(
        root: &Path,
        sm: &git2::Submodule,
        repo: &git2::Repository,
        offline: bool,
    ) -> Result<Submodule, Error> {
        let name = sm.name().unwrap_or("unknown").to_string();
        let sm_path = sm.path();
        let full_sm_path = root.join(sm_path);
        let url = sm.url().unwrap_or("").to_string();
        let branch = sm.branch().unwrap_or("main").to_string();

        let raw_status = repo.submodule_status(&name, git2::SubmoduleIgnore::None)?;
        let uninitialized = raw_status.is_wd_uninitialized();
        let parent_pointer = CommitHash(sm.head_id().unwrap_or_else(git2::Oid::zero).to_string());

        let rs = Self::remote_state(
            &full_sm_path,
            &branch,
            &parent_pointer,
            uninitialized,
            offline,
        );

        let dirty = !uninitialized
            && rs.ahead == 0
            && (raw_status.is_wd_modified()
                || raw_status.is_index_modified()
                || raw_status.is_wd_untracked());

        let status = Self::determine_status(StatusInput {
            uninitialized,
            dirty,
            detached: rs.detached,
            orphaned: rs.orphaned,
            remote_unreachable: rs.remote_unreachable,
            ahead: rs.ahead,
            behind: rs.behind,
            local_head: rs.local_head.clone(),
            parent_pointer: parent_pointer.clone(),
        });

        Ok(Submodule {
            name,
            path: sm_path.to_path_buf(),
            url,
            tracked_branch: branch,
            parent_pointer,
            local_head: rs.local_head,
            remote_head: rs.remote_head,
            status,
            ahead_count: rs.ahead,
            behind_count: rs.behind,
            remote_unreachable: rs.remote_unreachable,
        })
    }

    fn remote_state(
        path: &Path,
        branch: &str,
        parent: &CommitHash,
        uninitialized: bool,
        offline: bool,
    ) -> RemoteState {
        if uninitialized {
            return Self::default_remote_state();
        }
        let Ok(sub) = git2::Repository::open(path) else {
            return Self::default_remote_state();
        };
        let (local, detached) = Self::sub_head(&sub);
        if !offline {
            Self::sub_fetch(&sub)
        }
        let (remote, unreachable) = Self::sub_remote_ref(&sub, branch);
        let (ahead, behind, orphaned) = Self::sub_diff(&sub, &local, parent, &remote, unreachable);
        RemoteState {
            local_head: local,
            remote_head: remote,
            detached,
            ahead,
            behind,
            orphaned,
            remote_unreachable: unreachable,
        }
    }

    fn default_remote_state() -> RemoteState {
        RemoteState {
            local_head: CommitHash::default(),
            remote_head: CommitHash::default(),
            detached: false,
            ahead: 0,
            behind: 0,
            orphaned: false,
            remote_unreachable: false,
        }
    }

    fn sub_head(sub: &git2::Repository) -> (CommitHash, bool) {
        let head = sub.head().ok();
        let local = head
            .as_ref()
            .and_then(|r| r.target())
            .map(|o| CommitHash(o.to_string()))
            .unwrap_or_default();
        (local, head.map(|r| !r.is_branch()).unwrap_or(false))
    }

    fn sub_fetch(sub: &git2::Repository) {
        let Ok(mut remote) = sub.find_remote("origin") else {
            return;
        };
        let mut opts = git2::FetchOptions::new();
        opts.download_tags(git2::AutotagOption::None);
        let mut cb = git2::RemoteCallbacks::new();
        cb.transfer_progress(|_| true);
        opts.remote_callbacks(cb);
        let _ = remote.fetch(
            &["+refs/heads/*:refs/remotes/origin/*"],
            Some(&mut opts),
            None,
        );
    }

    fn sub_remote_ref(sub: &git2::Repository, branch: &str) -> (CommitHash, bool) {
        let refname = format!("refs/remotes/origin/{}", branch);
        match sub.find_reference(&refname).ok().and_then(|r| r.target()) {
            Some(oid) => (CommitHash(oid.to_string()), false),
            None => (CommitHash::default(), true),
        }
    }

    fn sub_diff(
        sub: &git2::Repository,
        local: &CommitHash,
        parent: &CommitHash,
        remote: &CommitHash,
        unreachable: bool,
    ) -> (usize, usize, bool) {
        let ahead = count_between(sub, parse_oid(parent), parse_oid(local));
        let behind = if unreachable {
            0
        } else {
            count_between(sub, parse_oid(local), parse_oid(remote))
        };
        let orphaned = !unreachable && remote != &CommitHash::default() && parent != remote && {
            let (p, r) = (parse_oid(parent), parse_oid(remote));
            match (p, r) {
                (Some(p_oid), Some(r_oid)) => sub
                    .merge_base(r_oid, p_oid)
                    .map(|b| b != p_oid)
                    .unwrap_or(true),
                _ => false,
            }
        };
        (ahead, behind, orphaned)
    }

    fn determine_status(input: StatusInput) -> SubmoduleStatus {
        if input.uninitialized {
            return SubmoduleStatus::Uninitialized;
        }
        if input.dirty {
            return SubmoduleStatus::Dirty;
        }
        if input.detached {
            return SubmoduleStatus::Detached;
        }
        if input.orphaned && !input.remote_unreachable {
            return SubmoduleStatus::Orphaned;
        }
        if (input.remote_unreachable && input.local_head != input.parent_pointer)
            || (input.ahead > 0 && input.behind == 0)
        {
            return SubmoduleStatus::AheadOfParent;
        }
        if input.behind > 0 && !input.remote_unreachable {
            return SubmoduleStatus::BehindRemote;
        }
        SubmoduleStatus::Clean
    }
}

// ── AggregateStatus ────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AggregateStatus {
    pub total: usize,
    pub clean: usize,
    pub ahead_of_parent: usize,
    pub behind_remote: usize,
    pub detached: usize,
    pub dirty: usize,
    pub orphaned: usize,
    pub uninitialized: usize,
}

impl AggregateStatus {
    pub fn from_submodules(submodules: &[Submodule]) -> Self {
        let mut s = AggregateStatus::default();
        s.total = submodules.len();
        for sm in submodules {
            match sm.status {
                SubmoduleStatus::Clean => s.clean += 1,
                SubmoduleStatus::AheadOfParent => s.ahead_of_parent += 1,
                SubmoduleStatus::BehindRemote => s.behind_remote += 1,
                SubmoduleStatus::Detached => s.detached += 1,
                SubmoduleStatus::Dirty => s.dirty += 1,
                SubmoduleStatus::Orphaned => s.orphaned += 1,
                SubmoduleStatus::Uninitialized => s.uninitialized += 1,
            }
        }
        s
    }
}

// ── GitSubmoduleEditor ─────────────────────────────────────────────────

/// Editor for syncing submodules. Generic over [`Git`] so callers can
/// inject a real or mock backend.
pub struct GitSubmoduleEditor<G: Git = RealGit> {
    root: PathBuf,
    offline: bool,
    _marker: PhantomData<G>,
}

impl<G: Git> GitSubmoduleEditor<G> {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            offline: false,
            _marker: PhantomData,
        }
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Sync a single submodule: fetch → rebase → push → update parent pointer → push parent.
    pub fn sync_to_parent(&self, name: &str) -> Result<(), Error> {
        let repo = git2::Repository::open(&self.root)?;
        let sm = repo
            .find_submodule(name)
            .map_err(|_| Error::NotFound(name.to_string()))?;
        let sm_path = sm.path();
        let full_sm_path = self.root.join(sm_path);

        if full_sm_path.exists() {
            if !self.offline {
                G::fetch(&full_sm_path)
            }
            Self::rebase_sub(&full_sm_path)?;
        }
        Self::push_sub(&full_sm_path)?;
        Self::update_parent_pointer(&repo, sm_path, name)?;
        if let Err(e) = Self::push_parent(&repo, &self.root) {
            revert_commit(&self.root);
            return Err(Error::Operation(format!(
                "父仓库 push 失败 (已回滚提交): {}",
                e
            )));
        }
        println!("  ✓ {}", name);
        Ok(())
    }

    /// Sync all submodules.
    pub fn sync_all_to_parent(&self) -> Result<(), Error> {
        let repo = git2::Repository::open(&self.root)?;
        let submodules = repo.submodules()?;
        println!("同步 {} 个子模块", submodules.len());
        for sm in &submodules {
            let name = sm.name().unwrap_or("unknown").to_string();
            match self.sync_to_parent(&name) {
                Ok(()) => {}
                Err(e) => println!("  {:<35} ✗ 失败: {}", name, e),
            }
        }
        Ok(())
    }

    /// Return health issues for all non-clean submodules.
    pub fn status(&self) -> Result<Vec<HealthIssue>, Error> {
        let state = RepoState::scan(&self.root)?;
        let mut issues = Vec::new();
        for sm in &state.submodules {
            if sm.status != SubmoduleStatus::Clean {
                let (desc, action) = describe_issue(sm.status);
                issues.push(HealthIssue {
                    submodule_name: sm.name.clone(),
                    status: format!("{:?}", sm.status),
                    description: desc,
                    suggested_action: action,
                });
            }
        }
        Ok(issues)
    }

    // ── low-level helpers (CLI operations) ──────────────────────────

    fn rebase_sub(path: &Path) -> Result<(), Error> {
        if !path.exists() {
            return Ok(());
        }
        let branch = G::current_branch(path).unwrap_or_default();
        if branch.is_empty() || branch == "HEAD" {
            return Ok(());
        }
        if !G::has_origin(path) {
            return Ok(());
        }

        let output = std::process::Command::new("git")
            .args(["rebase", &format!("origin/{}", branch)])
            .current_dir(path)
            .output()
            .map_err(|e| Error::Operation(format!("git rebase 无法执行: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.contains("up to date") || stderr.contains("up-to-date") {
                return Ok(());
            }
            return Err(Error::Operation(format!(
                "rebase 冲突，需手动处理：解决冲突后 git rebase --continue，或 git rebase --abort 放弃\n{}",
                stderr
            )));
        }
        Ok(())
    }

    fn push_sub(path: &Path) -> Result<(), Error> {
        if !path.exists() {
            return Ok(());
        }
        let branch = G::current_branch(path).unwrap_or_default();
        if branch.is_empty() || branch == "HEAD" {
            return Ok(());
        }
        if !G::has_origin(path) {
            return Ok(());
        }

        if G::ahead_count(path, &branch) == 0 {
            return Ok(());
        }

        let output = std::process::Command::new("git")
            .args(["push", "origin", &branch])
            .current_dir(path)
            .output()
            .map_err(|e| Error::Operation(format!("git push 无法执行: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(Error::Operation(format!("git push 失败: {}", stderr)));
        }
        Ok(())
    }

    fn update_parent_pointer(
        repo: &git2::Repository,
        sm_path: &Path,
        name: &str,
    ) -> Result<(), Error> {
        let mut index = repo.index()?;
        index.add_path(sm_path)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?;
        let parent = head.peel_to_commit()?;
        let sig = repo.signature()?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("chore: 更新子模块 '{}' 指针", name),
            &tree,
            &[&parent],
        )?;
        Ok(())
    }

    fn push_parent(repo: &git2::Repository, root: &Path) -> Result<(), String> {
        if !G::has_origin(root) {
            return Ok(());
        }
        let branch = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(String::from))
            .unwrap_or_default();
        if branch.is_empty() {
            return Err("无法检测当前分支".into());
        }
        let output = std::process::Command::new("git")
            .args(["push", "origin", &branch])
            .current_dir(root)
            .output()
            .map_err(|e| format!("git push 无法执行: {}", e))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
}

// ── Diagnostics ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub submodule_name: String,
    pub status: String,
    pub description: String,
    pub suggested_action: String,
}

fn describe_issue(status: SubmoduleStatus) -> (String, String) {
    match status {
        SubmoduleStatus::AheadOfParent => (
            "本地领先于父仓库记录".into(),
            "运行 sync_to_parent 更新父仓库指针".into(),
        ),
        SubmoduleStatus::BehindRemote => (
            "远程有更新，本地落后".into(),
            "运行 code sync 获取最新代码".into(),
        ),
        SubmoduleStatus::Detached => (
            "处于游离 HEAD 状态".into(),
            "运行 checkout_branch 切换到跟踪分支".into(),
        ),
        SubmoduleStatus::Dirty => ("有未提交的修改".into(), "提交或 stash 当前修改".into()),
        SubmoduleStatus::Orphaned => (
            "父仓库记录的 commit 在远程已不存在".into(),
            "需手动干预".into(),
        ),
        SubmoduleStatus::Uninitialized => ("尚未初始化".into(), "运行 init 初始化子模块".into()),
        SubmoduleStatus::Clean => unreachable!("describe_issue called on Clean"),
    }
}

// ── Git utilities ──────────────────────────────────────────────────────

fn parse_oid(h: &CommitHash) -> Option<git2::Oid> {
    git2::Oid::from_str(&h.0).ok()
}

fn count_between(repo: &git2::Repository, from: Option<git2::Oid>, to: Option<git2::Oid>) -> usize {
    let (Some(from), Some(to)) = (from, to) else {
        return 0;
    };
    if from == to {
        return 0;
    }
    let Ok(mut walk) = repo.revwalk() else {
        return 0;
    };
    let _ = walk.push(to);
    let _ = walk.hide(from);
    walk.count()
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git_init(path: &Path) {
        let repo = git2::Repository::init(path).unwrap();
        repo.set_head("refs/heads/main").unwrap();
    }

    fn git_commit(path: &Path, msg: &str) {
        let repo = git2::Repository::open(path).unwrap();
        std::fs::write(path.join("file"), msg).unwrap();
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("file")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs)
            .unwrap();
    }

    fn setup_repo_with_submodule(tmp: &Path) -> PathBuf {
        let parent = tmp.join("parent");
        let sub = tmp.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        git_init(&sub);
        git_commit(&sub, "init sub");
        std::fs::create_dir_all(&parent).unwrap();
        git_init(&parent);
        git_commit(&parent, "init parent");
        // git submodule add 没有 git2 等价操作
        std::process::Command::new("git")
            .args(["submodule", "add", &sub.to_string_lossy(), "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "add submodule"])
            .current_dir(&parent)
            .output()
            .unwrap();
        parent
    }

    // ── SubmoduleStatus ─────────────────────────────────────────────

    #[test]
    fn test_status_priority_ordering() {
        assert!(SubmoduleStatus::Dirty.priority() < SubmoduleStatus::Clean.priority());
        assert!(SubmoduleStatus::Orphaned.priority() < SubmoduleStatus::BehindRemote.priority());
    }

    #[test]
    fn test_clean_is_lowest_priority() {
        for s in &[
            SubmoduleStatus::Dirty,
            SubmoduleStatus::Orphaned,
            SubmoduleStatus::Detached,
            SubmoduleStatus::Uninitialized,
            SubmoduleStatus::BehindRemote,
            SubmoduleStatus::AheadOfParent,
        ] {
            assert!(s.priority() < SubmoduleStatus::Clean.priority());
        }
    }

    #[test]
    fn test_all_priorities_are_unique() {
        let all = [
            SubmoduleStatus::Dirty,
            SubmoduleStatus::Orphaned,
            SubmoduleStatus::Detached,
            SubmoduleStatus::Uninitialized,
            SubmoduleStatus::BehindRemote,
            SubmoduleStatus::AheadOfParent,
            SubmoduleStatus::Clean,
        ];
        let p: Vec<u8> = all.iter().map(|s| s.priority()).collect();
        let mut s = p.clone();
        s.sort();
        s.dedup();
        assert_eq!(p.len(), s.len());
    }

    #[test]
    fn test_status_debug_output() {
        assert_eq!(format!("{:?}", SubmoduleStatus::Clean), "Clean");
    }
    #[test]
    fn test_status_clone_eq() {
        assert_eq!(SubmoduleStatus::Dirty, SubmoduleStatus::Dirty);
    }

    // ── determine_status ────────────────────────────────────────────

    fn si(
        u: bool,
        di: bool,
        de: bool,
        o: bool,
        ur: bool,
        a: usize,
        b: usize,
        l: CommitHash,
        p: CommitHash,
    ) -> StatusInput {
        StatusInput {
            uninitialized: u,
            dirty: di,
            detached: de,
            orphaned: o,
            remote_unreachable: ur,
            ahead: a,
            behind: b,
            local_head: l,
            parent_pointer: p,
        }
    }
    fn dh() -> CommitHash {
        CommitHash::default()
    }
    fn ch(s: &str) -> CommitHash {
        CommitHash(s.to_string())
    }

    #[test]
    fn test_determine_status_uninitialized() {
        assert_eq!(
            RepoState::determine_status(si(true, false, false, false, false, 0, 0, dh(), dh())),
            SubmoduleStatus::Uninitialized
        );
    }
    #[test]
    fn test_determine_status_dirty() {
        assert_eq!(
            RepoState::determine_status(si(false, true, false, false, false, 0, 0, dh(), dh())),
            SubmoduleStatus::Dirty
        );
    }
    #[test]
    fn test_determine_status_detached() {
        assert_eq!(
            RepoState::determine_status(si(false, false, true, false, false, 0, 0, dh(), dh())),
            SubmoduleStatus::Detached
        );
    }
    #[test]
    fn test_determine_status_orphaned() {
        assert_eq!(
            RepoState::determine_status(si(false, false, false, true, false, 0, 0, dh(), dh())),
            SubmoduleStatus::Orphaned
        );
    }

    #[test]
    fn test_determine_status_ahead_of_parent() {
        assert_eq!(
            RepoState::determine_status(si(
                false,
                false,
                false,
                false,
                true,
                0,
                0,
                ch("abc"),
                dh()
            )),
            SubmoduleStatus::AheadOfParent
        );
        assert_eq!(
            RepoState::determine_status(si(false, false, false, false, false, 5, 0, dh(), dh())),
            SubmoduleStatus::AheadOfParent
        );
        assert_eq!(
            RepoState::determine_status(si(false, false, false, false, false, 5, 3, dh(), dh())),
            SubmoduleStatus::BehindRemote
        );
    }

    #[test]
    fn test_determine_status_behind_remote() {
        assert_eq!(
            RepoState::determine_status(si(false, false, false, false, false, 0, 3, dh(), dh())),
            SubmoduleStatus::BehindRemote
        );
        assert_eq!(
            RepoState::determine_status(si(false, false, false, false, true, 0, 3, dh(), dh())),
            SubmoduleStatus::Clean
        );
    }

    #[test]
    fn test_determine_status_clean() {
        assert_eq!(
            RepoState::determine_status(si(false, false, false, false, false, 0, 0, dh(), dh())),
            SubmoduleStatus::Clean
        );
    }

    // ── CommitHash ──────────────────────────────────────────────────

    #[test]
    fn test_commit_hash_display_truncates() {
        assert_eq!(CommitHash("abcdef1234567890".into()).to_string(), "abcdef1");
    }
    #[test]
    fn test_commit_hash_display_short() {
        assert_eq!(CommitHash("abc".into()).to_string(), "abc");
    }
    #[test]
    fn test_commit_hash_display_empty() {
        assert_eq!(CommitHash(String::new()).to_string(), "");
    }
    #[test]
    fn test_commit_hash_equality() {
        assert_eq!(CommitHash("abc".into()), CommitHash("abc".into()));
    }
    #[test]
    fn test_commit_hash_default() {
        assert_eq!(
            CommitHash::default().0,
            "0000000000000000000000000000000000000000"
        );
    }
    #[test]
    fn test_commit_hash_clone() {
        let a = CommitHash("deadbeef".into());
        assert_eq!(a, a.clone());
    }

    // ── Submodule ───────────────────────────────────────────────────

    #[test]
    fn test_submodule_fields() {
        let sm = Submodule {
            name: "test".into(),
            path: PathBuf::from("libs/test"),
            url: "https://example.com/test.git".into(),
            tracked_branch: "main".into(),
            parent_pointer: CommitHash("aaa".into()),
            local_head: CommitHash("bbb".into()),
            remote_head: CommitHash("ccc".into()),
            status: SubmoduleStatus::BehindRemote,
            ahead_count: 0,
            behind_count: 3,
            remote_unreachable: false,
        };
        assert_eq!(sm.name, "test");
    }

    // ── AggregateStatus ─────────────────────────────────────────────

    #[test]
    fn test_aggregate_status_default() {
        assert_eq!(AggregateStatus::default().total, 0);
    }

    #[test]
    fn test_aggregate_status_from_submodules() {
        let sm = |s| Submodule {
            name: String::new(),
            path: PathBuf::new(),
            url: String::new(),
            tracked_branch: "main".into(),
            parent_pointer: CommitHash::default(),
            local_head: CommitHash::default(),
            remote_head: CommitHash::default(),
            status: s,
            ahead_count: 0,
            behind_count: 0,
            remote_unreachable: false,
        };
        let agg = AggregateStatus::from_submodules(&[
            sm(SubmoduleStatus::Clean),
            sm(SubmoduleStatus::Dirty),
            sm(SubmoduleStatus::Orphaned),
        ]);
        assert_eq!(agg.total, 3);
        assert_eq!(agg.clean, 1);
        assert_eq!(agg.dirty, 1);
        assert_eq!(agg.orphaned, 1);
    }

    // ── parse_oid / count_between ───────────────────────────────────

    #[test]
    fn test_parse_oid_valid() {
        assert!(
            parse_oid(&CommitHash(
                "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0".into()
            ))
            .is_some()
        );
    }
    #[test]
    fn test_parse_oid_invalid() {
        assert!(parse_oid(&CommitHash("not-a-hex-string".into())).is_none());
    }
    #[test]
    fn test_parse_oid_empty() {
        assert!(parse_oid(&CommitHash(String::new())).is_none());
    }
    #[test]
    fn test_count_between_both_none() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        let r = git2::Repository::open(t.path()).unwrap();
        assert_eq!(count_between(&r, None, None), 0);
    }
    #[test]
    fn test_count_between_equal_oids() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        git_commit(t.path(), "c1");
        let r = git2::Repository::open(t.path()).unwrap();
        let h = r.head().unwrap().target().unwrap();
        assert_eq!(count_between(&r, Some(h), Some(h)), 0);
    }
    #[test]
    fn test_count_between_from_to() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        git_commit(t.path(), "c1");
        let r = git2::Repository::open(t.path()).unwrap();
        let c1 = r.head().unwrap().target().unwrap();
        git_commit(t.path(), "c2");
        let c2 = r.head().unwrap().target().unwrap();
        assert_eq!(count_between(&r, Some(c1), Some(c2)), 1);
    }
    #[test]
    fn test_count_between_revwalk_fail() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        let r = git2::Repository::open(t.path()).unwrap();
        let o = git2::Oid::from_str("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0").ok();
        assert_eq!(count_between(&r, o, o), 0);
    }

    // ── scan ────────────────────────────────────────────────────────

    #[test]
    fn test_scan_no_gitmodules() {
        assert!(RepoState::scan(&tempfile::tempdir().unwrap().path()).is_err());
    }
    #[test]
    fn test_scan_git_repo_but_no_submodules() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        git_commit(t.path(), "initial");
        assert_eq!(RepoState::scan(t.path()).unwrap().total, 0);
    }
    #[test]
    fn test_scan_non_git_directory() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join(".gitmodules"), "").unwrap();
        assert!(RepoState::scan(t.path()).is_err());
    }
    #[test]
    fn test_scan_with_submodule() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        let s = RepoState::scan(&p).unwrap();
        assert_eq!(s.total, 1);
        assert_eq!(s.submodules[0].name, "libs/sub");
    }
    #[test]
    fn test_scan_all_no_gitmodules() {
        assert!(RepoState::scan_all(&tempfile::tempdir().unwrap().path()).is_err());
    }
    #[test]
    fn test_scan_all_with_submodule() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        let (subs, _) = RepoState::scan_all(&p).unwrap();
        assert_eq!(subs.len(), 1);
    }
    #[test]
    fn test_repo_state_manual() {
        let s = RepoState {
            root_path: PathBuf::from("/tmp"),
            submodules: vec![],
            total: 0,
            clean_count: 0,
            needs_attention: vec![],
        };
        assert_eq!(s.total, 0);
    }

    // ── GitSubmoduleEditor ──────────────────────────────────────────

    #[test]
    fn test_editor_new_and_root() {
        let e = GitSubmoduleEditor::<RealGit>::new(PathBuf::from("/tmp"));
        assert_eq!(e.root(), Path::new("/tmp"));
    }

    #[test]
    fn test_editor_sync_to_parent() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        assert!(
            GitSubmoduleEditor::<RealGit>::new(p)
                .sync_to_parent("libs/sub")
                .is_ok()
        );
    }

    #[test]
    fn test_editor_sync_to_parent_nonexistent() {
        let t = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(t.path().join(".git")).unwrap();
        assert!(
            GitSubmoduleEditor::<RealGit>::new(t.path().to_path_buf())
                .sync_to_parent("no-such-module")
                .is_err()
        );
    }

    #[test]
    fn test_editor_sync_all_to_parent() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        assert!(
            GitSubmoduleEditor::<RealGit>::new(p)
                .sync_all_to_parent()
                .is_ok()
        );
    }

    #[test]
    fn test_editor_sync_all_to_parent_no_submodules() {
        let t = tempfile::tempdir().unwrap();
        git_init(t.path());
        git_commit(t.path(), "initial");
        assert!(
            GitSubmoduleEditor::<RealGit>::new(t.path().to_path_buf())
                .sync_all_to_parent()
                .is_ok()
        );
    }

    #[test]
    fn test_editor_status() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        assert!(
            GitSubmoduleEditor::<RealGit>::new(p)
                .status()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_editor_status_with_gitmodules_but_no_repo() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join(".gitmodules"), "").unwrap();
        assert!(
            GitSubmoduleEditor::<RealGit>::new(t.path().to_path_buf())
                .status()
                .is_err()
        );
    }

    #[test]
    fn test_editor_sync_with_remote_push() {
        let tmp = tempfile::tempdir().unwrap();
        let bare_sub = tmp.path().join("bare-sub");
        let bare_parent = tmp.path().join("bare-parent");
        for b in [&bare_sub, &bare_parent] {
            git2::Repository::init_bare(b).unwrap();
        }
        let sub = tmp.path().join("sub");
        // git clone 需要 credential 回调，测试用 CLI 更简洁
        Command::new("git")
            .args(["clone", &bare_sub.to_string_lossy(), &sub.to_string_lossy()])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        git_init(&sub);
        git_commit(&sub, "init");
        // git push 无交互时 CLI 比 git2 简单
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        let parent = tmp.path().join("parent");
        std::fs::create_dir_all(&parent).unwrap();
        git_init(&parent);
        git_commit(&parent, "init parent");
        {
            let repo = git2::Repository::open(&parent).unwrap();
            repo.remote("origin", &bare_parent.to_string_lossy())
                .unwrap();
        }
        // git submodule add 没有 git2 等价
        Command::new("git")
            .args(["submodule", "add", &bare_sub.to_string_lossy(), "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add submodule"])
            .current_dir(&parent)
            .output()
            .unwrap();
        git_commit(&sub, "ahead");
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        {
            let sub_repo = git2::Repository::open(&parent.join("libs/sub")).unwrap();
            let mut remote = sub_repo.find_remote("origin").unwrap();
            let mut opts = git2::FetchOptions::new();
            let mut cb = git2::RemoteCallbacks::new();
            cb.transfer_progress(|_| true);
            opts.remote_callbacks(cb);
            let _ = remote.fetch(
                &["+refs/heads/*:refs/remotes/origin/*"],
                Some(&mut opts),
                None,
            );
        }
        assert!(
            GitSubmoduleEditor::<RealGit>::new(parent)
                .sync_to_parent("libs/sub")
                .is_ok(),
            "sync failed"
        );
    }

    #[test]
    fn test_editor_sync_rebase_catches_up() {
        let tmp = tempfile::tempdir().unwrap();
        let bare_sub = tmp.path().join("bare-sub");
        let bare_parent = tmp.path().join("bare-parent");
        for b in [&bare_sub, &bare_parent] {
            git2::Repository::init_bare(b).unwrap();
        }
        let sub = tmp.path().join("sub");
        Command::new("git")
            .args(["clone", &bare_sub.to_string_lossy(), &sub.to_string_lossy()])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        git_init(&sub);
        git_commit(&sub, "init");
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        let init_hash = {
            let repo = git2::Repository::open(&sub).unwrap();
            repo.head().unwrap().target().unwrap().to_string()
        };
        let parent = tmp.path().join("parent");
        std::fs::create_dir_all(&parent).unwrap();
        git_init(&parent);
        git_commit(&parent, "init parent");
        {
            let repo = git2::Repository::open(&parent).unwrap();
            repo.remote("origin", &bare_parent.to_string_lossy())
                .unwrap();
        }
        Command::new("git")
            .args(["submodule", "add", &bare_sub.to_string_lossy(), "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add submodule"])
            .current_dir(&parent)
            .output()
            .unwrap();
        let sm_path = parent.join("libs/sub");
        {
            let repo = git2::Repository::open(&sm_path).unwrap();
            assert_eq!(
                repo.head().unwrap().target().unwrap().to_string(),
                init_hash,
                "submodule starts at init"
            );
        }
        git_commit(&sub, "remote ahead");
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        let remote_hash = {
            let repo = git2::Repository::open(&sub).unwrap();
            repo.head().unwrap().target().unwrap().to_string()
        };
        assert!(
            GitSubmoduleEditor::<RealGit>::new(parent)
                .sync_to_parent("libs/sub")
                .is_ok(),
            "sync failed"
        );
        {
            let repo = git2::Repository::open(&sm_path).unwrap();
            assert_eq!(
                repo.head().unwrap().target().unwrap().to_string(),
                remote_hash,
                "submodule caught up to remote after sync"
            );
        }
    }

    #[test]
    fn test_editor_status_with_dirty_submodule() {
        let t = tempfile::tempdir().unwrap();
        let p = setup_repo_with_submodule(t.path());
        std::fs::write(p.join("libs/sub/new-file"), "content").unwrap();
        let issues = GitSubmoduleEditor::<RealGit>::new(p).status().unwrap();
        assert!(!issues.is_empty());
        assert_eq!(issues[0].status, "Dirty");
    }

    // ── describe_issue ──────────────────────────────────────────────

    #[test]
    fn test_describe_issue_ahead_of_parent() {
        let (d, a) = describe_issue(SubmoduleStatus::AheadOfParent);
        assert!(d.contains("领先"));
        assert!(a.contains("sync"));
    }
    #[test]
    fn test_describe_issue_behind_remote() {
        let (d, a) = describe_issue(SubmoduleStatus::BehindRemote);
        assert!(d.contains("落后"));
        assert!(a.contains("sync"));
    }
    #[test]
    fn test_describe_issue_detached() {
        let (d, a) = describe_issue(SubmoduleStatus::Detached);
        assert!(d.contains("游离"));
        assert!(a.contains("checkout"));
    }
    #[test]
    fn test_describe_issue_dirty() {
        let (d, _a) = describe_issue(SubmoduleStatus::Dirty);
        assert!(d.contains("修改"));
    }
    #[test]
    fn test_describe_issue_orphaned() {
        let (d, _a) = describe_issue(SubmoduleStatus::Orphaned);
        assert!(d.contains("不存在"));
    }
    #[test]
    fn test_describe_issue_uninitialized() {
        let (d, _a) = describe_issue(SubmoduleStatus::Uninitialized);
        assert!(d.contains("初始化"));
    }
    #[test]
    #[should_panic(expected = "unreachable")]
    fn test_describe_issue_clean_panics() {
        describe_issue(SubmoduleStatus::Clean);
    }

    // ── edge case scan tests ────────────────────────────────────────

    #[test]
    fn test_scan_with_uninitialized_submodule() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path().join("parent");
        std::fs::create_dir_all(&parent).unwrap();
        git_init(&parent);
        git_commit(&parent, "init");
        let sub = tmp.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        git_init(&sub);
        git_commit(&sub, "init");
        Command::new("git")
            .args(["submodule", "add", &sub.to_string_lossy(), "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add submodule"])
            .current_dir(&parent)
            .output()
            .unwrap();
        Command::new("git")
            .args(["submodule", "deinit", "-f", "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        assert_eq!(
            RepoState::scan(&parent).unwrap().submodules[0].status,
            SubmoduleStatus::Uninitialized
        );
    }

    #[test]
    fn test_scan_with_detached_submodule() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let sm_path = parent.join("libs/sub");
        {
            let sub_repo = git2::Repository::open(&sm_path).unwrap();
            let oid = sub_repo.head().unwrap().target().unwrap();
            sub_repo.set_head_detached(oid).unwrap();
        }
        assert_eq!(
            RepoState::scan(&parent).unwrap().submodules[0].status,
            SubmoduleStatus::Detached
        );
    }

    #[test]
    fn test_scan_with_ahead_via_remote_unreachable() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let sm_path = parent.join("libs/sub");
        std::fs::write(sm_path.join("new-file"), "content").unwrap();
        {
            let sub_repo = git2::Repository::open(&sm_path).unwrap();
            let sig = git2::Signature::now("test", "test@test.com").unwrap();
            let mut index = sub_repo.index().unwrap();
            index.add_path(Path::new("new-file")).unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = sub_repo.find_tree(tree_id).unwrap();
            let parent_commit = sub_repo.head().ok().and_then(|h| h.peel_to_commit().ok());
            let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
            sub_repo
                .commit(Some("HEAD"), &sig, &sig, "ahead commit", &tree, &parents)
                .unwrap();
            sub_repo.remote_delete("origin").unwrap();
        }
        let state = RepoState::scan(&parent).unwrap();
        assert_eq!(state.submodules[0].status, SubmoduleStatus::AheadOfParent);
    }

    #[test]
    fn test_scan_with_subrepo_open_error() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let sm_git = parent.join("libs/sub/.git");
        if sm_git.is_dir() {
            std::fs::remove_dir_all(&sm_git).unwrap();
        } else {
            std::fs::remove_file(&sm_git).unwrap();
        }
        assert_eq!(
            RepoState::scan(&parent).unwrap().submodules[0].local_head,
            CommitHash::default()
        );
    }

    #[test]
    fn test_scan_with_behind_remote() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path().join("parent");
        let sub = tmp.path().join("sub");
        let bare = tmp.path().join("bare");
        std::fs::create_dir_all(&bare).unwrap();
        git2::Repository::init_bare(&bare).unwrap();
        Command::new("git")
            .args(["clone", &bare.to_string_lossy(), &sub.to_string_lossy()])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        git_init(&sub);
        git_commit(&sub, "init");
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        std::fs::create_dir_all(&parent).unwrap();
        git_init(&parent);
        git_commit(&parent, "init parent");
        Command::new("git")
            .args(["submodule", "add", &sub.to_string_lossy(), "libs/sub"])
            .current_dir(&parent)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add submodule"])
            .current_dir(&parent)
            .output()
            .unwrap();
        git_commit(&sub, "remote ahead");
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&sub)
            .output()
            .unwrap();
        {
            let sub_repo = git2::Repository::open(&parent.join("libs/sub")).unwrap();
            let mut remote = sub_repo.find_remote("origin").unwrap();
            let mut opts = git2::FetchOptions::new();
            let mut cb = git2::RemoteCallbacks::new();
            cb.transfer_progress(|_| true);
            opts.remote_callbacks(cb);
            let _ = remote.fetch(
                &["+refs/heads/*:refs/remotes/origin/*"],
                Some(&mut opts),
                None,
            );
        }
        assert_eq!(
            RepoState::scan(&parent).unwrap().submodules[0].behind_count,
            1
        );
    }

    #[test]
    fn test_scan_with_orphaned_submodule() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let sm_path = parent.join("libs/sub");
        {
            let sub_repo = git2::Repository::open(&sm_path).unwrap();
            sub_repo.remote_delete("origin").unwrap();
        }
        let ref_dir = parent.join(".git/modules/libs/sub/refs/remotes/origin");
        std::fs::create_dir_all(&ref_dir).unwrap();
        std::fs::write(
            ref_dir.join("main"),
            "1111111111111111111111111111111111111111\n",
        )
        .unwrap();
        assert_eq!(
            RepoState::scan(&parent).unwrap().submodules[0].status,
            SubmoduleStatus::Orphaned
        );
    }

    #[test]
    fn test_scan_with_ahead_of_parent_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        git_commit(&parent.join("libs/sub"), "ahead commit");
        assert!(RepoState::scan(&parent).unwrap().submodules[0].ahead_count > 0);
    }

    #[test]
    fn test_orphaned_parse_oid_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let ref_dir = parent.join(".git/modules/libs/sub/refs/remotes/origin");
        if !ref_dir.exists() {
            std::fs::create_dir_all(&ref_dir).unwrap();
        }
        std::fs::write(ref_dir.join("main"), "not-a-valid-oid\n").unwrap();
        assert!(!RepoState::scan(&parent).unwrap().submodules.is_empty());
    }

    #[test]
    fn test_ahead_of_parent_via_ahead_count() {
        let tmp = tempfile::tempdir().unwrap();
        let parent = setup_repo_with_submodule(tmp.path());
        let sm_path = parent.join("libs/sub");
        {
            let sub_repo = git2::Repository::open(&sm_path).unwrap();
            sub_repo.remote_delete("origin").unwrap();
            std::fs::write(sm_path.join("new-file"), "content").unwrap();
            let sig = git2::Signature::now("test", "test@test.com").unwrap();
            let mut index = sub_repo.index().unwrap();
            index.add_path(Path::new("new-file")).unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = sub_repo.find_tree(tree_id).unwrap();
            let parent_commit = sub_repo.head().ok().and_then(|h| h.peel_to_commit().ok());
            let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
            sub_repo
                .commit(Some("HEAD"), &sig, &sig, "ahead", &tree, &parents)
                .unwrap();
        }
        let state = RepoState::scan(&parent).unwrap();
        assert_eq!(state.submodules[0].ahead_count, 1);
    }
}
