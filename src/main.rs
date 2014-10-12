extern crate getopts;
extern crate git2;

use std::os;
use std::io::fs::PathExtensions;
use getopts::{optflag,getopts};
use git2::{Repository,Commit,Oid};

fn is_git_repo(path: &Path) -> bool {
    let abs_path: Path = os::make_absolute(path).join(".git");

    return abs_path.exists() && abs_path.is_dir();
}

fn handle_commit(commits: &mut Vec<Oid>, commit: &Commit) {
    let id: Oid = commit.id();
    if !commits.contains(&id) {
        commits.push(id);
        for parent in commit.parents() {
            handle_commit(commits, &parent);
        }
    }
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    println!("{}", program);

    let opts = [
        optflag("h", "help", "print this help menu")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_string()) }
    };

    let input = if !matches.free.is_empty() {
        println!("count of opts: {}", matches.free.len());
        matches.free[0].clone()
    } else {
        println!("usage");
        return;
    };

    let path = Path::new(input.as_slice());

    if is_git_repo(&path) {
        let repo = match Repository::open(&path) {
            Ok(repo) => repo,
            Err(e) => fail!("failed to open `{}`: {}", path.display(), e),
        };

        let oid_commit = match repo.head() {
            Ok(reference) => reference.target().unwrap(),
            Err(e) => fail!("{}", e)
        };

        let commit = match repo.find_commit(oid_commit) {
            Ok(commit) => commit,
            Err(e) => fail!("{}", e)
        };

        let mut commits: Vec<Oid> = Vec::new();
        handle_commit(&mut commits, &commit);

        println!("Total count of commits: {}", commits.len());
    }
}

