use git2;
use std::{fmt,path};
use collections::vec::IntoIter;
use std::collections::BTreeMap;

pub struct Snapshot {
    files: Vec<path::PathBuf>,
    extensions: BTreeMap<String, usize>
}

impl Snapshot {

    pub fn new(repo: &git2::Repository, id: git2::Oid) -> Result<Snapshot, git2::Error> {
        Snapshot::create(repo, id, "")
    }

    fn create(repo: &git2::Repository, id: git2::Oid, prefix: &str) -> Result<Snapshot, git2::Error> {
        let mut files: Vec<path::PathBuf> = Vec::new();
        let mut extensions: BTreeMap<String, usize> = BTreeMap::new();

        let head_object: git2::Object = try!(repo.find_object(id, Some(git2::ObjectType::Tree)));
        let mut trees: Vec<git2::Object> = vec![head_object];

        while let Some(object) = trees.pop() {
            for entry in object.as_tree().unwrap().iter() {
                // println!("{} {}", entry.id(), path.to_str().unwrap());
                match entry.kind() {
                    Some(git2::ObjectType::Tree) => {
                        trees.push(try!(entry.to_object(repo)));
                    },
                    Some(git2::ObjectType::Blob) => {
                        if let Some(name) = entry.name() {
                            // TODO: calculate full path
                            let path = path::PathBuf::from(prefix).join(name);

                            let ext = match path.extension() {
                                Some(ext) => {
                                    let ext_str = match ext.to_str() {
                                        Some(ext_str) => ext_str,
                                        None => "none"
                                    };
                                    String::from_str(ext_str)
                                },
                                None => String::from_str("none")
                            };

                            *extensions.entry(ext).or_insert(0) += 1;

                            files.push(path)
                        }
                    },
                    _ => {}
                }
            }
        }

        Ok(Snapshot {
            files: files,
            extensions: extensions
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
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

        for (ext, count) in self.extensions.iter() {
            let value = (*count) as f32 * step;

            if value < 1_f32 || *ext == "none" {
                other += value
            } else {
                for _ in 0..(value.ceil() as usize) {
                    let _ = write!(f, "{}", ARTS[pos % ARTS.len()]);
                }
                labels.push(&ext[..]);
                pos += 1;
            }
        }

        for _ in 0..(other.ceil() as usize) {
            let _ = write!(f, "{}", ARTS[pos % ARTS.len()]);
        }

        labels.push("other");
        let _ = write!(f, "\n{:?}\n", labels);

        Ok(())
    }
}
