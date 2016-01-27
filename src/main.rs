extern crate git2;
extern crate chrono;
extern crate docopt;
extern crate rustc_serialize;
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
    use std::cmp;
    use std::path::Path;
    use std::collections::BTreeMap;
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
        let mut authors = PersonalStats::new(&repo);
        let mut num_files: BTreeMap<String, usize> = BTreeMap::new();

        for (i, commit) in commits.iter().enumerate() {

            print!("[{}/{}]\r", i+1, commits.len());

            heatmap.append(&commit.author().when());
            try!(authors.append(&commit, mailmap));

            let files = try!(Snapshot::new(&repo, &commit));
            let key = format!("{}", files.datetime.format("%Y-%W"));
            let number = num_files.entry(key).or_insert(0);
            *number = cmp::max(*number, files.len());
        }
        println!("");

        let mut vec: Vec<usize> = num_files.values().cloned().collect();
        vec.sort_by(|a, b| b.cmp(a));
        let max = cmp::max(1, vec[0]);

        const WIDTH: usize = 60;

        let coeff = if max > WIDTH {
            WIDTH as f32 / max as f32
        } else {
            1f32
        };

        println!("Files in repo:");
        for (key, val) in num_files {
            let value = (val as f32 * coeff).round() as usize;
            let bar = (0..value).map(|_| "░").collect::<String>();
            println!("{} {:3} {}", key, val, bar + "▏");
        }
        println!("");

        println!("{}", heatmap);
        println!("{}", authors);

        Ok(())
    }

}
