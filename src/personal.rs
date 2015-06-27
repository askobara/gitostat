use std::fmt;
use git2;

pub struct PersonalStat {
    count: usize,
    insertions: usize,
    deletions: usize,
}

impl PersonalStat {
    pub fn new() -> PersonalStat {
        PersonalStat { count: 0, insertions: 0, deletions: 0 }
    }

    pub fn add(&mut self, rhs: &git2::DiffStats) -> &PersonalStat {
        self.count += 1;
        self.insertions += rhs.insertions();
        self.deletions += rhs.deletions();
        self
    }
}

impl fmt::Display for PersonalStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} +{} -{}", self.count, self.insertions, self.deletions)
    }
}

