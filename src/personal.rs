use std::{fmt,ops,cmp};
use std::error::Error;
use std::ops::{Add, AddAssign};
use std::collections::{BTreeMap, HashMap};
use git2;
use chrono;
use chrono::offset::{FixedOffset, Utc, Local, TimeZone};
use mailmap::Mailmap;
use snapshot::Snapshot;
use prettytable::{Table, format};

pub struct PersonalStats<'repo> {
    repo: &'repo git2::Repository,
    authors: HashMap<String, Stat>,
}

impl<'repo> PersonalStats<'repo> {
    pub fn new(repo: &'repo git2::Repository) -> PersonalStats<'repo> {
        PersonalStats { repo: repo, authors: HashMap::new() }
    }

    pub fn append(&mut self, commit: &git2::Commit, mailmap: Option<&Mailmap>) -> Result<(), git2::Error> {
        let name = PersonalStats::mapped_name(&commit.author(), mailmap)?;

        let stat = self.repo.stat(&commit)?;

        *self.authors.entry(name).or_insert(Stat::new()) += stat;
        Ok(())
    }

    pub fn blame(&mut self, files: &Snapshot, mailmap: Option<&Mailmap>) -> Result<(), git2::Error> {
        let mut opts = git2::BlameOptions::new();
        opts.track_copies_same_commit_moves(true)
            .track_copies_same_commit_copies(true);

        for (i, path) in files.iter().enumerate() {
            print!("[{}/{}]\r", i+1, files.len());

            let blame = self.repo.blame_file(path, Some(&mut opts))?;

            for hunk in blame.iter() {
                let name = PersonalStats::mapped_name(&hunk.final_signature(), mailmap)?;

                if let Some(entry) = self.authors.get_mut(&name) {
                    entry.num_lines += hunk.lines_in_hunk();
                }
            }
        }

        Ok(())
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
        let format = format::FormatBuilder::new()
                .column_separator('│')
                .borders('│')
                .separator(format::LinePosition::Top,    format::LineSeparator::new('─', '┬', '┌', '┐'))
                .separator(format::LinePosition::Intern, format::LineSeparator::new('─', '┼', '├', '┤'))
                .separator(format::LinePosition::Bottom, format::LineSeparator::new('─', '┴', '└', '┘'))
                .padding(1, 1)
                .build();

        table.set_format(format);
        table.add_row(row!["Author", "Commits (%)", "Insertions", "Deletions", "Owned lines (%)", "Live code", "Age in days", "Active days (%)"]);

        let total = self.authors.iter().fold(Stat::new(), |total, item| total + item.1);
        let total_days = total.num_days();
        let total_active_days = total.activity_days.len();
        let total_active_days_percent = total_active_days as f32 / total_days as f32 * 100_f32;
        let total_live_code_percent = total.num_lines as f32 / total.insertions as f32 * 100_f32;

        for (name, stat) in &self.authors {
            let active_days = stat.activity_days.len();
            let all_days = cmp::max(1, stat.num_days());
            let active_days_percent = active_days as f32 / all_days as f32 * 100_f32;
            let commit_percent = stat.num_commit as f32 / total.num_commit as f32 * 100_f32;
            let lines_percent = stat.num_lines as f32 / total.num_lines as f32 * 100_f32;
            let live_code_percent = stat.num_lines as f32 / stat.insertions as f32 * 100_f32;

            table.add_row(row![
                          name,
                          format!("{} ({:.2}%)", stat.num_commit, commit_percent),
                          format!("{}", stat.insertions),
                          format!("{}", stat.deletions),
                          format!("{} ({:.2}%)", stat.num_lines, lines_percent),
                          format!("{:.2}%", live_code_percent),
                          format!("{}", all_days),
                          format!("{} ({:.2}%)", active_days, active_days_percent)
            ]);
        }

        table.add_row(row![
                      "Total",
                      format!("{} (100%)", total.num_commit),
                      format!("{}", total.insertions),
                      format!("{}", total.deletions),
                      format!("{} (100%)", total.num_lines),
                      format!("{:.2}%", total_live_code_percent),
                      format!("{}", total_days),
                      format!("{} ({:.2}%)", total_active_days, total_active_days_percent)
        ]);

        let mut vec: Vec<usize> = total.activity_weeks.values().cloned().collect();
        vec.sort_by(|a, b| b.cmp(a));
        let max = cmp::max(1, vec[0]);

        const WIDTH: usize = 60;

        let coeff = if max > WIDTH {
            WIDTH as f32 / max as f32
        } else {
            1f32
        };

        let now = Local::now();
        let start = total.first_commit.clone().unwrap();
        let num_weeks = now.signed_duration_since(start.datetime).num_weeks();

        writeln!(f, "Activity by weeks:")?;
        for i in 0..num_weeks {
            let step = start.datetime.add(chrono::Duration::weeks(i));
            let key = format!("{}", step.format("%Y-%W"));
            let val = *total.activity_weeks.get(&key).unwrap_or(&0);
            let value = (val as f32 * coeff).round() as usize;
            let bar = (0..value).map(|_| "░").collect::<String>();
            writeln!(f, "{} {:3} {}", key, val, bar + "▏")?;
        }
        writeln!(f, "")?;

        write!(f, "{}", table)
    }
}

#[derive(Copy, Clone, Debug)]
struct MiniCommit {
    id: git2::Oid,
    datetime: chrono::DateTime<FixedOffset>,
}

impl MiniCommit {
    pub fn new(commit: &git2::Commit) -> MiniCommit {
        let time = commit.author().when();
        let tz = FixedOffset::east(time.offset_minutes() * 60);

        MiniCommit {
            id: commit.id(),
            datetime: Utc.timestamp(time.seconds(), 0).with_timezone(&tz),
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
pub struct Stat {
    num_commit: usize,
    num_lines: usize,
    insertions: usize,
    deletions: usize,

    activity_days: HashMap<String, usize>,
    activity_weeks: BTreeMap<String, usize>,

    last_commit: Option<MiniCommit>,
    first_commit: Option<MiniCommit>,
}

pub trait HasStat {
    fn stat(&self, commit: &git2::Commit) -> Result<Stat, git2::Error>;
}

impl HasStat for git2::Repository {
    fn stat(&self, commit: &git2::Commit) -> Result<Stat, git2::Error> {

        let mini = MiniCommit::new(commit);
        let tree = commit.tree()?;

        // avoid error on the initial commit
        let ptree = if commit.parents().len() == 1 {
            let parent = commit.parent(0)?;
            parent.tree().ok()
        } else {
            None
        };

        let diff = self.diff_tree_to_tree(ptree.as_ref(), Some(&tree), None)?;
        let stats = diff.stats()?;

        let mut activity_days = HashMap::new();
        let day = format!("{}", mini.datetime.format("%Y-%m-%d"));
        activity_days.insert(day, 1);

        let mut activity_weeks = BTreeMap::new();
        let week = format!("{}", mini.datetime.format("%Y-%W"));
        activity_weeks.insert(week, 1);

        Ok(Stat {
            num_commit: 1,
            num_lines: 0,
            insertions: stats.insertions(),
            deletions: stats.deletions(),

            activity_days: activity_days,
            activity_weeks: activity_weeks,
            first_commit: Some(mini),
            last_commit: Some(mini),
        })
    }
}

impl Stat {
    /// Create empty struct.
    pub fn new() -> Stat {
        Stat {
            num_commit: 0,
            num_lines: 0,
            insertions: 0,
            deletions: 0,
            activity_days: HashMap::new(),
            activity_weeks: BTreeMap::new(),
            first_commit: None,
            last_commit: None,
        }
    }

    /// Returns number of days between first and last commits.
    pub fn num_days(&self) -> i64 {
        let first = self.first_commit.clone().unwrap();
        let last = self.last_commit.clone().unwrap();

        last.datetime.signed_duration_since(first.datetime).num_days()
    }
}


fn merge_hashmaps(lhs: &HashMap<String, usize>, rhs: &HashMap<String, usize>) -> HashMap<String, usize> {
    let mut result = lhs.clone();
    for (key, value) in rhs.iter() {
        *result.entry(key.clone()).or_insert(0) += *value;
    }

    result
}

fn merge_btreemaps(lhs: &BTreeMap<String, usize>, rhs: &BTreeMap<String, usize>) -> BTreeMap<String, usize> {
    let mut result = lhs.clone();
    for (key, value) in rhs.iter() {
        *result.entry(key.clone()).or_insert(0) += *value;
    }

    result
}

impl<'a> ops::Add<&'a Stat> for Stat {
    type Output = Stat;

    fn add(self, rhs: &'a Stat) -> Stat {
        Stat {
            num_commit: self.num_commit + rhs.num_commit,
            num_lines: self.num_lines + rhs.num_lines,
            insertions: self.insertions + rhs.insertions,
            deletions: self.deletions + rhs.deletions,
            activity_days: merge_hashmaps(&self.activity_days, &rhs.activity_days),
            activity_weeks: merge_btreemaps(&self.activity_weeks, &rhs.activity_weeks),
            // because None is smaller than other.datetime
            first_commit: if self.first_commit.is_none() { rhs.first_commit } else { cmp::min(self.first_commit, rhs.first_commit) },
            last_commit: cmp::max(self.last_commit, rhs.last_commit)
        }
    }
}

impl AddAssign for Stat {
    fn add_assign(&mut self, rhs: Stat) {
        self.num_commit += rhs.num_commit;
        self.num_lines += rhs.num_lines;
        self.insertions += rhs.insertions;
        self.deletions += rhs.deletions;
        for (key, value) in &rhs.activity_days {
            *self.activity_days.entry(key.clone()).or_insert(0) += *value;
        }
        for (key, value) in &rhs.activity_weeks {
            *self.activity_weeks.entry(key.clone()).or_insert(0) += *value;
        }
        // because None is smaller than other.datetime
        self.first_commit = if self.first_commit.is_none() {
            rhs.first_commit
        } else {
            cmp::min(self.first_commit, rhs.first_commit)
        };

        self.last_commit = cmp::max(self.last_commit, rhs.last_commit);
    }
}
