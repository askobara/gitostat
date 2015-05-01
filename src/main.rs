#![feature(os)]
#![feature(core)]
#![feature(collections)]
// #![feature(unicode)]
#![feature(str_char)]

extern crate git2;
extern crate chrono;
extern crate getopts;
// extern crate unicode;
extern crate docopt;
extern crate rustc_serialize;
extern crate collections;
extern crate core;

use docopt::Docopt;

mod files;
mod heatmap;

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

    match gitstat::run(&args) {
        Ok(()) => {},
        Err(e) => println!("error: {}", e)
    }
}

mod gitstat {

    use git2;
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;
    use std::sync::{Mutex, Arc};
    use std::path::Path;
    use core::iter::IntoIterator;
    use std::ffi;
    use Args;

    use files::Files;
    use heatmap::Heatmap;

    pub fn run(args: &Args) -> Result<(), git2::Error> {
        let path = Path::new(&args.arg_path);
        let repo = try!(git2::Repository::open(path));

        let authors: HashMap<String, usize> = try!(self::get_authors(&repo));

        // iterate over everything.
        for (name, count) in authors.iter() {
            println!("{}: {}", *name, *count);
        }

        Ok(())
    }

    /// Helper method for gets HEAD commit of given git repository
    fn get_head_commit(repo: &Box<git2::Repository>) -> Option<git2::Commit> {
        repo.head().ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok())
    }

    fn get_authors(repo: &git2::Repository) -> Result<HashMap<String, usize>, git2::Error> {
        let mut heatmap = Heatmap::new();
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());
        // let mut files_number = Vec::new();

        revwalk.push_head();
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        let oids: Vec<git2::Oid> = revwalk.by_ref().collect();
        println!("total count: {}", oids.len());

        for oid in oids[..10].iter() {
            let commit = try!(repo.find_commit(*oid));
            let mut extensions: HashMap<ffi::OsString, usize> = HashMap::new();
            heatmap.append(&commit.time());

            let tree = try!(commit.tree());

            let files = try!(Files::new(&repo, &tree));

            for path in files.iter() {
                // println!("{}", path.display());

                if let Some(ext) = path.extension() {
                    match extensions.entry(ext.to_os_string()) {
                        Entry::Vacant(entry) => entry.insert(1),
                        Entry::Occupied(mut entry) => {
                            *entry.get_mut() += 1;
                            entry.into_mut()
                        }
                    };
                }
            }
            println!("{} {}", oid, files.len());
            println!("{:?}\n", extensions);

            let uniq_name: String = get_uniq_name(&commit.author());

            match authors.entry(uniq_name) {
                Entry::Vacant(entry) => entry.insert(1),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() += 1;
                    entry.into_mut()
                }
            };
        }

        println!("{}", heatmap);

        Ok(authors)
    }

    fn get_uniq_name(author: &git2::Signature) -> String {
        format!("{} <{}>", author.name().unwrap(), author.email().unwrap())
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
