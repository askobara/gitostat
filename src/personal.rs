use std::{fmt,ops,cmp,default};
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
        let name = try!(PersonalStats::mapped_name(&commit.author(), mailmap));

        self.authors
            .entry(name)
            .or_insert(Stat::new(&commit))
            .calculate(self.repo, &commit)
    }

    pub fn mapped_name(sig: &git2::Signature, mailmap: Option<&Mailmap>) -> Result<String, git2::Error> {
        match mailmap {
            None => Ok(format!("{}", sig)),
            Some(mm) => {
                mm.map_user(&sig)
                     .map_err(|err| git2::Error::from_str(err.description()))
            }
        }
    }
}

impl<'repo> fmt::Display for PersonalStats<'repo> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = Table::new();
        table.add_row(row!["Author", "Commits (%)", "Insertions", "Deletions", "Age in days", "Active days (%)"]);

        let mut total = Stat::default();

        for (name, stat) in self.authors.iter() {
            total = &total + stat;

            let active_days = stat.activity.len();
            let all_days = cmp::max(1, stat.num_days());
            let active_days_percent = active_days as f32 / all_days as f32 * 100_f32;
            let commit_percent = stat.num_commit as f32 / self.total as f32 * 100_f32;

            table.add_row(row![
                          name,
                          format!("{} ({:.2}%)", stat.num_commit, commit_percent),
                          format!("{}", stat.insertions),
                          format!("{}", stat.deletions),
                          format!("{}", all_days),
                          format!("{} ({:.2}%)", active_days, active_days_percent)
            ]);
        }

        let total_days = total.num_days();
        let total_active_days = total.activity.len();
        let total_active_days_percent = total_active_days as f32 / total_days as f32 * 100_f32;
        table.add_row(row![
                      "Total",
                      format!("{} (100%)", total.num_commit),
                      format!("{}", total.insertions),
                      format!("{}", total.deletions),
                      format!("{}", total_days),
                      format!("{} ({:.2}%)", total_active_days, total_active_days_percent)
        ]);

        write!(f, "{}", table)
    }
}

#[derive(Copy, Clone, Debug)]
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

impl cmp::Ord for MiniCommit {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl cmp::PartialOrd for MiniCommit {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.datetime.cmp(&other.datetime))
    }
}

impl cmp::PartialEq for MiniCommit {
    fn eq(&self, other: &Self) -> bool {
        self.datetime == other.datetime
    }
}

impl cmp::Eq for MiniCommit { }

#[derive(Debug)]
struct Stat {
    num_commit: usize,
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
            num_commit: 0,
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

        self.num_commit += 1;
        self.insertions += stats.insertions();
        self.deletions += stats.deletions();
        self.first_commit = Some(mini);
        Ok(())
    }

    /// Returns number of days between first and last commits.
    pub fn num_days(&self) -> i64 {
        let first = self.first_commit.clone().unwrap();
        let last = self.last_commit.clone().unwrap();

        (last.datetime - first.datetime).num_days()
    }
}


impl default::Default for Stat {
    fn default() -> Stat {
        Stat {
            num_commit: 0,
            insertions: 0,
            deletions: 0,
            activity: HashMap::new(),
            first_commit: None,
            last_commit: None
        }
    }
}

impl<'a, 'b> ops::Add<&'b Stat> for &'a Stat {
    type Output = Stat;

    fn add(self, rhs: &'b Stat) -> Stat {
        let mut activity = self.activity.clone();
        for (key, value) in rhs.activity.iter() {
            *activity.entry(key.clone()).or_insert(0) += *value;
        }

        Stat {
            num_commit: self.num_commit + rhs.num_commit,
            insertions: self.insertions + rhs.insertions,
            deletions: self.deletions + rhs.deletions,
            activity: activity,
            // because None is smaller than other.datetime
            first_commit: if self.first_commit.is_none() { rhs.first_commit } else { cmp::min(self.first_commit, rhs.first_commit) },
            last_commit: cmp::max(self.last_commit, rhs.last_commit)
        }
    }
}
