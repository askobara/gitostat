use std::fmt;
use git2;
use chrono;
use chrono::offset::{fixed,utc,TimeZone};
use std::collections::HashMap;
use mailmap::Mailmap;
use prettytable::Table;
use std::error::Error;

pub struct PersonalStats<'repo> {
    repo: &'repo git2::Repository,
    authors: HashMap<String, Stat>,
    total: usize,
}

impl<'repo> PersonalStats<'repo> {
    pub fn new(repo: &'repo git2::Repository, total: usize) -> PersonalStats<'repo> {
        PersonalStats { repo: repo, authors: HashMap::new(), total: total }
    }

    pub fn append(&mut self, commit: &git2::Commit, mailmap: Option<&Mailmap>) -> Result<(), git2::Error> {
        let name: String = match mailmap {
            None => format!("{}", commit.author()),
            Some(mm) => {
                try!(mm.map_user(&commit.author())
                     .map_err(|err| git2::Error::from_str(err.description())))
            }
        };

        self.authors
            .entry(name)
            .or_insert(Stat::new(&commit))
            .calculate(self.repo, &commit)
    }
}

impl<'repo> fmt::Display for PersonalStats<'repo> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = Table::new();
        table.add_row(row!["Author", "Commits (%)", "Insertions", "Deletions", "Age", "Active days"]);

        for (name, stat) in self.authors.iter() {
            let percent = stat.count as f32 / self.total as f32 * 100_f32;
            let first = stat.first_commit.clone().unwrap();
            let last = stat.last_commit.clone().unwrap();

            table.add_row(row![
                          name,
                          format!("{} ({:.2}%)", stat.count, percent),
                          format!("{}", stat.insertions),
                          format!("{}", stat.deletions),
                          format!("{}", (last.datetime - first.datetime).num_days()),
                          format!("{}", stat.activity.len())
            ]);
        }

        write!(f, "{}", table)
    }
}

#[derive(Clone, Copy)]
struct MiniCommit {
    id: git2::Oid,
    datetime: chrono::datetime::DateTime<chrono::offset::fixed::FixedOffset>,
}

impl MiniCommit {
    pub fn new(commit: &git2::Commit) -> MiniCommit {
        let time = commit.time();
        let tz = fixed::FixedOffset::east(time.offset_minutes() * 60);

        MiniCommit {
            id: commit.id(),
            datetime: utc::UTC.timestamp(time.seconds(), 0).with_timezone(&tz),
        }
    }
}

struct Stat {
    count: usize,
    insertions: usize,
    deletions: usize,

    activity: HashMap<String, usize>,

    last_commit: Option<MiniCommit>,
    first_commit: Option<MiniCommit>,
}

impl Stat {
    /// Create empty struct.
    pub fn new(commit: &git2::Commit) -> Stat {
        Stat {
            count: 0,
            insertions: 0,
            deletions: 0,
            activity: HashMap::new(),
            first_commit: None,
            last_commit: Some(MiniCommit::new(commit)),
        }
    }

    /// Gets diff and save it for current stats.
    pub fn calculate(&mut self, repo: &git2::Repository, commit: &git2::Commit) -> Result<(), git2::Error> {
        let mini = MiniCommit::new(commit);
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

        // counting active days
        let date = format!("{}", mini.datetime.format("%Y-%m-%d"));
        *self.activity.entry(date).or_insert(0) += 1;

        self.count += 1;
        self.insertions += stats.insertions();
        self.deletions += stats.deletions();
        self.first_commit = Some(mini);
        Ok(())
    }
}

