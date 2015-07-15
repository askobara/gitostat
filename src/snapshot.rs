use git2;
use std::{fmt,path,slice};
use collections::vec::IntoIter;
use std::collections::BTreeMap;
use chrono::offset::{fixed,utc,TimeZone};
use chrono::datetime;

pub struct Snapshot {
    files: Vec<path::PathBuf>,
    extensions: BTreeMap<String, usize>,
    datetime: datetime::DateTime<fixed::FixedOffset>
}

impl Snapshot {

    pub fn new(repo: &git2::Repository, commit: &git2::Commit) -> Result<Snapshot, git2::Error> {
        let mut files: Vec<path::PathBuf> = Vec::new();
        let mut extensions: BTreeMap<String, usize> = BTreeMap::new();

        let head_object: git2::Object = try!(repo.find_object(commit.tree_id(), Some(git2::ObjectType::Tree)));
        let mut trees: Vec<(path::PathBuf, git2::Object)> = vec![(path::PathBuf::new(), head_object)];

        while let Some((path, object)) = trees.pop() {
            // gets all entries of tree
            for entry in object.as_tree().unwrap().iter() {
                match entry.kind() {
                    // the other trees will be added to the stack with
                    // calculated path
                    Some(git2::ObjectType::Tree) => {
                        let name = entry.name().unwrap_or("<non-utf8 string>");
                        trees.push((path.join(name), try!(entry.to_object(repo))));
                    },
                    // blob (aka file) will be pushed to result vector
                    Some(git2::ObjectType::Blob) => {
                        if let Some(name) = entry.name() {
                            // TODO: calculate full path
                            let path = path.join(name);
                            let ext = match path.extension() {
                                Some(ext) => {
                                    String::from(ext.to_str().unwrap_or("none"))
                                },
                                None => String::from("none")
                            };

                            *extensions.entry(ext).or_insert(0) += 1;

                            files.push(path)
                        }
                    },
                    _ => {}
                }
            }
        }

        let time = commit.author().when();
        let tz = fixed::FixedOffset::east(time.offset_minutes() * 60);
        let datetime = utc::UTC.timestamp(time.seconds(), 0) .with_timezone(&tz);

        Ok(Snapshot {
            files: files,
            extensions: extensions,
            datetime: datetime
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn iter(&self) -> slice::Iter<path::PathBuf> {
        self.files.iter()
    }
}

impl IntoIterator for Snapshot {
    type Item = path::PathBuf;
    type IntoIter = IntoIter<path::PathBuf>;

    fn into_iter(self) -> IntoIter<path::PathBuf> {
        self.files.into_iter()
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const MAX: usize = 80;
        const ARTS: [char; 4] = ['░', '▒', '▓', '█'];

        let step = MAX as f32 / self.files.len() as f32 ;
        let mut pos = 0;
        let mut other = 0_f32;
        let mut labels: Vec<&str> = Vec::new();

        try!(writeln!(f, "{}", self.datetime));

        // TODO: sort by count
        for (ext, count) in self.extensions.iter() {
            let value = (*count) as f32 * step;

            if value < 1_f32 || *ext == "none" {
                other += value
            } else {
                for _ in 0..(value.ceil() as usize) {
                    try!(write!(f, "{}", ARTS[pos % ARTS.len()]));
                }
                labels.push(&ext[..]);
                pos += 1;
            }
        }

        for _ in 0..(other.ceil() as usize) {
            try!(write!(f, "{}", ARTS[pos % ARTS.len()]));
        }

        labels.push("other");

        write!(f, "\n{:?}\n", labels)
    }
}
