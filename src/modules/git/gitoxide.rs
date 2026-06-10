use std::path::Path;

use gix::{
    Repository, bstr::BString, progress::Discard, remote::Direction, status::Item, status::index_worktree::Item as WtItem,
    status::plumbing::index_as_worktree::EntryStatus,
};

use super::GitStats;

pub fn run_git(path: &Path) -> GitStats {
    let Ok(repo) = gix::open(path) else {
        return empty("Big Bang".into());
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

fn head_info(repo: &Repository) -> (String, u32, u32) {
    let mut head_ref = match repo.head_ref() {
        Ok(Some(r)) => r,
        _ => {
            let name = repo
                .head_id()
                .ok()
                .and_then(|id| id.shorten().ok().map(|s| s.to_string()))
                .unwrap_or_else(|| "Big Bang".into());
            return (name, 0, 0);
        },
    };

    let branch_name = head_ref.name().shorten().to_string();

    let local = match head_ref.peel_to_id() {
        Ok(id) => id.detach(),
        Err(_) => return (branch_name, 0, 0),
    };

    let Some(Ok(upstream_name)) = head_ref.remote_tracking_ref_name(Direction::Fetch) else {
        return (branch_name, 0, 0);
    };
    let Ok(mut upstream_ref) = repo.find_reference(upstream_name.as_ref()) else {
        return (branch_name, 0, 0);
    };
    let upstream = match upstream_ref.peel_to_id() {
        Ok(id) => id.detach(),
        Err(_) => return (branch_name, 0, 0),
    };

    let ahead = count_walk(repo, local, upstream);
    let behind = count_walk(repo, upstream, local);
    (branch_name, ahead, behind)
}

fn count_walk(repo: &Repository, tip: gix::ObjectId, hidden: gix::ObjectId) -> u32 {
    repo.rev_walk([tip])
        .with_hidden([hidden])
        .all()
        .map(|w| w.filter_map(Result::ok).count() as u32)
        .unwrap_or(0)
}

fn empty(branch_name: String) -> GitStats {
    GitStats { untracked: 0, conflicted: 0, non_staged: 0, staged: 0, ahead: 0, behind: 0, branch_name }
}
