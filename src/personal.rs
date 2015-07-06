use std::fmt;
use git2;
use chrono;
use chrono::offset::{fixed,utc,TimeZone};

pub struct PersonalStat {
    count: usize,
    insertions: usize,
    deletions: usize,

    last_commit: Option<MiniCommit>,
    first_commit: Option<MiniCommit>,
}

#[derive(Clone, Copy)]
struct MiniCommit {
    id: git2::Oid,
    time: chrono::datetime::DateTime<chrono::offset::fixed::FixedOffset>,
}

impl MiniCommit {
    pub fn new(commit: &git2::Commit) -> MiniCommit {
        let time = commit.time();
        let tz = fixed::FixedOffset::east(time.offset_minutes() * 60);

        MiniCommit {
            id: commit.id(),
            time: utc::UTC.timestamp(time.seconds(), 0).with_timezone(&tz),
        }
    }
}

impl PersonalStat {

    /// Create empty struct.
    pub fn new(commit: &git2::Commit) -> PersonalStat {
        PersonalStat {
            count: 0,
            insertions: 0,
            deletions: 0,
            first_commit: None,
            last_commit: Some(MiniCommit::new(commit)),
        }
    }

    /// Gets diff and save it for current stats.
    pub fn calculate(&mut self, repo: &git2::Repository, commit: &git2::Commit) -> Result<(), git2::Error> {
        let tree = try!(commit.tree());

        // avoid error on the initial commit
        let ptree = if commit.parents().len() == 1 {
            let parent = try!(commit.parent(0));
            Some(try!(parent.tree()))
        } else {
            None
        };

        let diff = try!(git2::Diff::tree_to_tree(repo, ptree.as_ref(), Some(&tree), None));
        let stats = try!(diff.stats());

        self.count += 1;
        self.insertions += stats.insertions();
        self.deletions += stats.deletions();
        self.first_commit = Some(MiniCommit::new(commit));
        Ok(())
    }
}

impl fmt::Display for PersonalStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first = self.first_commit.clone().unwrap();
        let last = self.last_commit.clone().unwrap();

        write!(f, "{} +{} -{}\nAge {} days",
               self.count, self.insertions, self.deletions,
               (last.time - first.time).num_days())
    }
}

