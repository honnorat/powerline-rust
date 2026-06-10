use std::path::Path;

use gix::{
    Repository, bstr::BString, progress::Discard, remote::Direction, status::Item, status::index_worktree::Item as WtItem,
    status::plumbing::index_as_worktree::EntryStatus,
};

use super::GitStats;

/// Collect working-tree + upstream state using `gix` (no subprocess).
pub fn run_git(path: &Path) -> GitStats {
    let Ok(repo) = gix::open(path) else {
        return GitStats::default();
    };

    let (mut untracked, mut non_staged, mut conflicted, mut staged) = (0u32, 0u32, 0u32, 0u32);

    if let Ok(platform) = repo.status(Discard) {
        if let Ok(iter) = platform.into_iter(std::iter::empty::<BString>()) {
            for item in iter.flatten() {
                match item {
                    Item::IndexWorktree(WtItem::Modification { status, .. }) => match status {
                        EntryStatus::Conflict { .. } => conflicted += 1,
                        EntryStatus::Change(_) => non_staged += 1,
                        EntryStatus::IntentToAdd | EntryStatus::NeedsUpdate(_) => {},
                    },
                    Item::IndexWorktree(WtItem::DirectoryContents { entry, .. }) => {
                        if matches!(entry.status, gix::dir::entry::Status::Untracked) {
                            untracked += 1;
                        }
                    },
                    Item::IndexWorktree(WtItem::Rewrite { .. }) => non_staged += 1,
                    Item::TreeIndex(_) => staged += 1,
                }
            }
        }
    }

    let (branch_name, ahead, behind) = head_info(&repo);

    GitStats { untracked, conflicted, non_staged, staged, ahead, behind, branch_name }
}

/// Resolve the current branch name plus `(ahead, behind)` vs its upstream.
///
/// Falls back to a short commit hash for detached HEAD, or "Big Bang" on an unborn branch.
fn head_info(repo: &Repository) -> (String, u32, u32) {
    let Ok(Some(mut head_ref)) = repo.head_ref() else {
        let name = repo
            .head_id()
            .ok()
            .and_then(|id| id.shorten().ok().map(|s| s.to_string()))
            .unwrap_or_else(|| "Big Bang".into());
        return (name, 0, 0);
    };

    let branch_name = head_ref.name().shorten().to_string();

    let Ok(local) = head_ref.peel_to_id() else { return (branch_name, 0, 0); };
    let Some(Ok(upstream_name)) = head_ref.remote_tracking_ref_name(Direction::Fetch) else {
        return (branch_name, 0, 0);
    };
    let Ok(mut upstream_ref) = repo.find_reference(upstream_name.as_ref()) else {
        return (branch_name, 0, 0);
    };
    let Ok(upstream) = upstream_ref.peel_to_id() else { return (branch_name, 0, 0); };

    let ahead = count_walk(repo, local.detach(), upstream.detach());
    let behind = count_walk(repo, upstream.detach(), local.detach());
    (branch_name, ahead, behind)
}

/// Count commits reachable from `tip` but excluding those reachable from `hidden`.
fn count_walk(repo: &Repository, tip: gix::ObjectId, hidden: gix::ObjectId) -> u32 {
    repo.rev_walk([tip])
        .with_hidden([hidden])
        .all()
        .map(|w| w.filter_map(Result::ok).count() as u32)
        .unwrap_or(0)
}
