#![feature(core)]
#![feature(collections)]
#![feature(plugin)]

#![plugin(regex_macros)]

extern crate git2;
extern crate chrono;
extern crate getopts;
extern crate docopt;
extern crate rustc_serialize;
extern crate collections;
extern crate core;
extern crate regex;

use docopt::Docopt;

mod snapshot;
mod heatmap;
mod mailmap;

#[derive(RustcDecodable)]
pub struct Args {
    arg_path: String
}

#[cfg(not(test))]
fn main() {
    const USAGE: &'static str = "
usage: gitostat [options] <path>
Options:
-h, --help show this message
";
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match gitostat::run(&args) {
        Ok(()) => {},
        Err(e) => println!("error: {}", e)
    }
}

macro_rules! error(
    ($($arg:tt)*) => (
        use std::io::Write;
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

#[macro_export]
/// converts errors into None
macro_rules! otry {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => {
            error!("ERROR!: {:?} {} {}", e, file!(), line!());
            return None
        }
    })
}

mod gitostat {

    use git2::{self, Diff};
    use std::collections::HashMap;
    use std::path::Path;
    use std::error::Error;
    use Args;

    use snapshot::Snapshot;
    use heatmap::Heatmap;
    use mailmap::Mailmap;

    pub fn run(args: &Args) -> Result<(), git2::Error> {
        let path = Path::new(&args.arg_path);
        let repo = try!(git2::Repository::open(path));

        let mailmap = try!(Mailmap::new(&path.join(".mailmap"))
                           .map_err(|err| git2::Error::from_str(err.description())));

        try!(self::info(&repo, &mailmap));

        Ok(())
    }

    fn info(repo: &git2::Repository, mailmap: &Mailmap) -> Result<(), git2::Error> {
        let mut heatmap = Heatmap::new();
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());

        try!(revwalk.push_head());
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        let commits = revwalk.filter_map(|oid| {
            // trying lookup commit in repo, skip if any error
            let commit = otry!(repo.find_commit(oid));

            // also skip merge-commits
            if commit.parents().len() > 1 { return None; }

            Some(commit)
        });

        // println!("total count: {}", oids.len());

        for commit in commits {
            let tree = try!(commit.tree());
            let ptree = if commit.parents().len() == 1 {
                let parent = try!(commit.parent(0));
                Some(try!(parent.tree()))
            } else {
                None
            };

            let diff = try!(Diff::tree_to_tree(&repo, ptree.as_ref(), Some(&tree), None));
            let stats = try!(diff.stats());

            heatmap.append(&commit.time());

            let files = try!(Snapshot::new(&repo, &commit));

            let uniq_name: String = try!(mailmap.map_user(&commit.author()).map_err(|err| git2::Error::from_str(err.description())));
            *authors.entry(uniq_name).or_insert(0) += 1;

            println!("{} {}", commit.id(), commit.author());
            println!("+/- {:4}/{:4}", stats.insertions(), stats.deletions());
            for path in files.iter() {
                println!("{}", path.display());
            }
            println!("Total files: {}\n {}", files.len(), files);

        }

        // let blame = try!(repo.blame_file(Path::new("web/index.php"), None));
        // for hunk in blame.iter() {
        //     println!("{} {}", hunk.final_commit_id(), hunk.final_signature());
        // }

        println!("{}", heatmap);

        // iterate over everything.
        for (name, count) in authors.iter() {
            println!("{}: {}", *name, *count);
        }

        Ok(())
    }

}
