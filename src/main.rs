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
    use chrono::offset::fixed::FixedOffset;
    use chrono::offset::utc::UTC;
    use chrono::offset::TimeZone;
    use chrono::{Datelike, Timelike};
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
                    _ => {}
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
        let mut heatmap = [0; 24*7];
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());
        // let mut files_number = Vec::new();

        revwalk.push_head();
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        for oid in revwalk {
            let commit = try!(repo.find_commit(oid));

            let uniq_name: String = get_uniq_name(&commit.author());
            let (weekday, hour) = get_heatmat_coords(&commit.time());
            let tree = try!(commit.tree());
            // let files = try!(Files::new(&repo, &tree));

            // files_number.push(files.len());

            heatmap[(weekday * 24 + hour) as usize] += 1;
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
        // TODO: sort this
        for count in heatmap.iter() {
            max = cmp::max(*count, max);
        }

        let arts = ['.', '▪', '◾', '◼', '⬛'];

        print!(" ");
        for i in 0..24 {
            print!("{:3}", i);
        }
        println!("");
        for i in 0..24*7 {
            if i % 24 == 0 {
                print!("{}: ", i / 24);
            }
            print!("{:3}", arts[(heatmap[i] as f32 / max as f32 * (arts.len() - 1) as f32) as usize]);
            // print!("{:3}", heatmap[i]);
            if (i + 1) % 24 == 0 {
                println!("");
            }
        }

        Ok(authors)
    }

    fn get_uniq_name(author: &Signature) -> String {
        format!("{} <{}>", author.name().unwrap(), author.email().unwrap())
    }

    fn get_heatmat_coords(time: &Time) -> (u32, u32) {
        let timestamp = UTC.timestamp(time.seconds(), 0)
            .with_timezone(&FixedOffset::east(time.offset_minutes() * 60));

        (timestamp.weekday().num_days_from_monday(), timestamp.hour())
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
