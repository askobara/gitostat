use git2;
use std::path::{PathBuf,Path};
use std::slice;
use collections::vec::IntoIter;

pub struct Snapshot {
    files: Vec<PathBuf>
}

impl Snapshot {

    pub fn new(repo: &git2::Repository, head: &git2::Tree) -> Result<Snapshot, git2::Error> {
        Snapshot::create(repo, head, "")
    }

    fn create(repo: &git2::Repository, head: &git2::Tree, prefix: &str) -> Result<Snapshot, git2::Error> {
        let mut vec = Vec::with_capacity(64);

        for entry in head.iter() {
            if let Some(name) = entry.name() {
                let path = PathBuf::from(prefix).join(name);

                // println!("{} {}", entry.id(), path.to_str().unwrap());
                match entry.kind() {
                    Some(git2::ObjectType::Tree) => {
                        let object: git2::Object = try!(entry.to_object(repo));

                        if let (Some(subtree), Some(subpath)) = (object.as_tree(), path.to_str()) {
                            let subfolder = try!(Snapshot::create(repo, &subtree, subpath));
                            vec.push_all(&subfolder.files);
                        }
                    },
                    Some(git2::ObjectType::Blob) => {
                        vec.push(path)
                    },
                    _ => {}
                }
            }
        }

        Ok(Snapshot {
            files: vec
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn iter(&self) -> slice::Iter<PathBuf> {
        self.files.iter()
    }
}

impl IntoIterator for Snapshot {
    type Item = PathBuf;
    type IntoIter = IntoIter<PathBuf>;

    fn into_iter(self) -> IntoIter<PathBuf> {
        self.files.into_iter()
    }
}

