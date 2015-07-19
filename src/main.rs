#![feature(core)]
#![feature(collections)]
#![feature(plugin)]

#![plugin(regex_macros)]

extern crate git2;
extern crate chrono;
extern crate docopt;
extern crate rustc_serialize;
extern crate collections;
extern crate core;
extern crate regex;
#[macro_use]
extern crate prettytable;

use docopt::Docopt;

mod snapshot;
mod heatmap;
mod mailmap;
mod personal;

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
/// converts errors into None and output them into stderr.
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
    use git2;
    use std::path::Path;
    use Args;

    use snapshot::Snapshot;
    use heatmap::Heatmap;
    use mailmap::Mailmap;
    use personal::PersonalStats;

    pub fn run(args: &Args) -> Result<(), git2::Error> {
        let path = Path::new(&args.arg_path);
        let repo = try!(git2::Repository::open(path));

        let mailmap = Mailmap::new(&path.join(".mailmap"));

        self::info(&repo, mailmap.as_ref())
    }

    fn info(repo: &git2::Repository, mailmap: Option<&Mailmap>) -> Result<(), git2::Error> {
        let mut revwalk = try!(repo.revwalk());
        try!(revwalk.push_head());
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        let commits: Vec<git2::Commit> = revwalk.filter_map(|oid| {
            // trying lookup commit in repo, skip if any error
            let commit = otry!(repo.find_commit(oid));
            // also skip merge-commits
            if commit.parents().len() > 1 { return None; }

            Some(commit)
        }).collect();

        let mut heatmap = Heatmap::new();
        let mut authors = PersonalStats::new(&repo, commits.len());

        for commit in commits.iter() {

            heatmap.append(&commit.author().when());
            try!(authors.append(&commit, mailmap));

            // println!("{} {}", commit.id(), uniq_name);
            //
            let files = try!(Snapshot::new(&repo, &commit));
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
        println!("{}", authors);

        Ok(())
    }

}
