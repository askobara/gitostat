use git2;
use std::{path,slice,marker};
use chrono::offset::{fixed,utc,TimeZone};
use chrono::datetime;

pub struct Snapshot<'repo> {
    files: Vec<path::PathBuf>,
    pub datetime: datetime::DateTime<fixed::FixedOffset>,
    _marker: marker::PhantomData<&'repo git2::Repository>,
}

/// Example:
///
/// ```
/// let files = try!(Snapshot::new(&repo, &commit));
/// for path in files.iter() {
///     println!("{}", path.display());
/// }
/// ```
impl<'repo> Snapshot<'repo> {

    pub fn new(repo: &'repo git2::Repository, commit: &git2::Commit, no_binary: bool) -> Result<Snapshot<'repo>, git2::Error> {
        let mut files: Vec<path::PathBuf> = Vec::new();

        let head = try!(repo.find_object(commit.tree_id(), Some(git2::ObjectType::Tree)));
        let mut trees = vec![(path::PathBuf::new(), head)];

        while let Some((path, object)) = trees.pop() {
            // gets all entries of tree
            for entry in object.as_tree().unwrap().iter() {
                match entry.kind() {
                    // other trees with resolved path will be added to the stack
                    Some(git2::ObjectType::Tree) => {
                        let name = entry.name().unwrap_or("<non-utf8 string>");
                        let object = try!(entry.to_object(repo));
                        trees.push((path.join(name), object));
                    },
                    // blob will be pushed to result vector
                    Some(git2::ObjectType::Blob) => {
                        if let Some(name) = entry.name() {
                            let path = path.join(name);

                            let is_binary = if no_binary {
                                let object = try!(entry.to_object(repo));
                                object.as_blob().unwrap().is_binary()
                            } else {
                                false
                            };

                            if !is_binary {
                                files.push(path)
                            }

                        }
                    },
                    _ => {}
                }
            }
        }

        let time = commit.author().when();
        let tz = fixed::FixedOffset::east(time.offset_minutes() * 60);
        let datetime = utc::UTC.timestamp(time.seconds(), 0).with_timezone(&tz);

        Ok(Snapshot {
            files: files,
            datetime: datetime,
            _marker: marker::PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> slice::Iter<path::PathBuf> {
        self.files.iter()
    }
}
