#![feature(os)]
#![feature(core)]
#![feature(collections)]
#![feature(unicode)]
#![feature(str_char)]

extern crate git2;
extern crate chrono;
extern crate getopts;
extern crate unicode;
extern crate docopt;
extern crate rustc_serialize;
extern crate collections;
extern crate core;

use std::path::Path;
use docopt::Docopt;

#[derive(RustcDecodable)]
pub struct Args {
    arg_path: String
}

#[cfg(not(test))]
fn main() {
    const USAGE: &'static str = "
usage: gitstat [options] <path>
Options:
-h, --help show this message
";
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    // println!("Total count of commits in '{}' is: {}", path.display(), stat.total);
    match gitstat::run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}

mod gitstat {
    use git2;
    use git2::{Repository,Commit,Oid,Signature,Time,Tree,Object,ObjectType};
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;
    use std::sync::{Mutex, Arc};
    use std::path::Path;
    use collections::vec::IntoIter;
    use core::iter::IntoIterator;
    use std::ops::Add;
    use Args;
    use chrono::datetime::DateTime;
    use chrono::naive::datetime::NaiveDateTime;
    use chrono::{Timelike, Datelike};
    use std::cmp;

    struct Files {
        files: Vec<String>
    }

    impl Files {

        pub fn new(repo: &Repository, head: &Tree) -> Result<Files, git2::Error> {
            let mut vec = Vec::new();

            for entry in head.iter() {
                let name: String = format!("{}", entry.name().unwrap());

                match entry.kind() {
                    Some(ObjectType::Tree) => {
                        let object: Object = try!(entry.to_object(repo));
                        let subfolder = try!(Files::new(repo, &object.as_tree().unwrap()));
                        vec.push_all(&subfolder.files);
                    },
                    Some(ObjectType::Blob) => vec.push(name),
                    _ => println!("{}", entry.kind().unwrap())
                }
            }

            Ok(Files {
                files: vec
            })
        }

        pub fn len(&self) -> usize {
            self.files.len()
        }
    }

    impl IntoIterator for Files {
        type Item = String;
        type IntoIter = IntoIter<String>;

        fn into_iter(self) -> IntoIter<String> {
            self.files.into_iter()
        }
    }

    pub fn run(args: &Args) -> Result<(), git2::Error> {
        let path = Path::new(&args.arg_path);
        let repo = try!(Repository::open(path));

        let authors: HashMap<String, usize> = try!(self::get_authors(&repo));

        // iterate over everything.
        for (name, count) in authors.iter() {
            println!("{}: {}", *name, *count);
        }

        Ok(())
    }

    /// Helper method for gets HEAD commit of given git repository
    fn get_head_commit(repo: &Box<Repository>) -> Option<Commit> {
        repo.head().ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok())
    }

    fn get_authors(repo: &Repository) -> Result<HashMap<String, usize>, git2::Error> {
        let mut heatmap = [[0u32; 24]; 7];
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());

        revwalk.push_head();
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        // mailmap::read_mailmap_file(path);
        for oid in revwalk {
            let commit = try!(repo.find_commit(oid));

            let uniq_name: String = get_uniq_name(&commit.author());
            let (weekday, hour) = get_heatmat_coords(&commit.time());
            let tree = try!(commit.tree());
            let files = try!(Files::new(&repo, &tree));

            // for item in files {
            //     println!("{}", item);
            // }
            println!("{} {}", oid, files.len());

            heatmap[weekday as usize][hour as usize] += 1;
            // println!("{} {}", time.seconds(), time.offset_minutes());

            match authors.entry(uniq_name) {
                Entry::Vacant(entry) => entry.insert(1),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() += 1;
                    entry.into_mut()
                }
            };
        }

        // find max
        let mut max: u32 = 0;
        for hours in heatmap.iter() {
            for count in hours.iter() {
                max = cmp::max(*count, max);
            }
        }

        let arts = ['.', '▪', '◾', '◼', '⬛'];

        for hours in heatmap.iter() {
            for count in hours.iter() {
                print!("{:4}", arts[(count % arts.len() as u32) as usize]);
                // print!("{:4}", count);
            }
            println!("");
        }

        Ok(authors)
    }

    fn get_uniq_name(author: &Signature) -> String {
        format!("{} <{}>", author.name().unwrap(), author.email().unwrap())
    }

    fn get_heatmat_coords(time: &Time) -> (u32, u32) {
        let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp(time.seconds(), 0);

        (timestamp.weekday().num_days_from_monday(), timestamp.hour())
    }

    /// Module `mailmap` which implement logic for parse .mailmap files
    mod mailmap {

        use std::env;
        use std::fs::File;
        use std::io::{BufReader, BufRead};
        use std::string::String;
        use std::path::Path;
        use unicode::str::UnicodeStr;

        pub fn read_mailmap_file(basedir: &Path) {
            // get full path to .mailmap
            let path = match env::current_dir() {
                Ok(path) => path.join(".mailmap"),
                Err(error) => panic!("{}", error),
            };

            // create BufReader instance for .mailmap file
            let mut reader = match File::open(&path) {
                Ok(file) => BufReader::new(file),
                Err(error) => panic!("{}", error),
            };

            for line in reader.lines() {
                match line {
                    Ok(line) => self::read_mailmap_line(line.as_slice()),
                    Err(x) => break
                };
            }
        }

        struct Author {
            name: String,
            email: String
        }

        struct MailmapLine {
            new_author: Author,
            old_author: Author
        }

        fn read_mailmap_line(line: &str) -> Option<MailmapLine> {
            if line.len() > 0 && line.char_at(0) != '#' {
                parse_mailmap_name_email(line);
            }

            None
        }

        fn parse_mailmap_name_email(line: &str) -> Option<Author> {
            let left = match line.find('<') {
                Some(i) => i,
                None => -1
            };

            let right = match line.find('>') {
                Some(i) => i,
                None => -1
            };

            // if left == -1 || right == -1 {
            //     return None;
            // }

            if left > right {
                return None;
            }

            if left+1 == right {
                return None;
            }

            Some(Author {
                name: String::from_str(UnicodeStr::trim(&line[..left])),
                email: String::from_str(UnicodeStr::trim(&line[left..right]))
            })

        }

        #[test]
        fn test_parse_name_email() {
            assert!(self::parse_mailmap_name_email("name <email>").is_some());
            assert!(self::parse_mailmap_name_email("name <>").is_none());
            assert!(self::parse_mailmap_name_email(">").is_none());
            assert!(self::parse_mailmap_name_email("<").is_none());
            assert!(self::parse_mailmap_name_email("><").is_none());
            assert!(self::parse_mailmap_name_email("").is_none());
            assert!(self::parse_mailmap_name_email("<email>").is_some());
        }

    }
}

#[cfg(test)]
mod tests {
    use std::old_io::TempDir;
    use git2::Repository;
    use gitstat::run;

    // fn repo_init() -> (TempDir, Repository) {
    //     let td = TempDir::new("test").unwrap();
    //     let repo = Repository::init(td.path()).unwrap();
    //     {
    //         let mut config = repo.config().unwrap();
    //         config.set_str("user.name", "name").unwrap();
    //         config.set_str("user.email", "email").unwrap();
    //         let mut index = repo.index().unwrap();
    //         let id = index.write_tree().unwrap();
    //
    //         let tree = repo.find_tree(id).unwrap();
    //         let sig = repo.signature().unwrap();
    //         repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, []).unwrap();
    //     }
    //     (td, repo)
    // }
    //
    // #[test]
    // fn smoke_run() {
    //     let (td, _repo) = self::repo_init();
    //     let path = td.unwrap();
    //     let result = run(&path);
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap().total, 1);
    // }
}
