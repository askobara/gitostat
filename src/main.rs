#![feature(core)]
#![feature(collections)]

extern crate git2;
extern crate chrono;
extern crate getopts;
// extern crate unicode;
extern crate docopt;
extern crate rustc_serialize;
extern crate collections;
extern crate core;

use docopt::Docopt;

mod snapshot;
mod heatmap;

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

mod gitostat {

    use git2;
    use std::collections::HashMap;
    use std::path::Path;
    use Args;

    use snapshot::Snapshot;
    use heatmap::Heatmap;

    pub fn run(args: &Args) -> Result<(), git2::Error> {
        let path = Path::new(&args.arg_path);
        let repo = try!(git2::Repository::open(path));

        try!(self::info(&repo));

        Ok(())
    }

    fn info(repo: &git2::Repository) -> Result<(), git2::Error> {
        let mut heatmap = Heatmap::new();
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());

        try!(revwalk.push_head());
        revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
        // let mutex = Mutex::new(repo);

        let oids: Vec<git2::Oid> = revwalk.by_ref().collect();
        println!("total count: {}", oids.len());

        for oid in oids[..].iter() {
            let commit = try!(repo.find_commit(*oid));
            heatmap.append(&commit.time());

            let files = try!(Snapshot::new(&repo, commit.tree_id()));

            // for path in files.iter() {
                // println!("{}", path.display());

            // }
            println!("{} {}", oid, files.len());
            println!("{}", files);

            let uniq_name: String = get_uniq_name(&commit.author());
            *authors.entry(uniq_name).or_insert(0) += 1;
        }

        println!("{}", heatmap);

        // iterate over everything.
        for (name, count) in authors.iter() {
            println!("{}: {}", *name, *count);
        }

        Ok(())
    }

    fn get_uniq_name(author: &git2::Signature) -> String {
        format!("{} <{}>", author.name().unwrap(), author.email().unwrap())
    }

}
