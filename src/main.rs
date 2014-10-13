extern crate getopts;
extern crate git2;

use std::os::{args};
use getopts::{optflag,getopts};

fn main() {
    let args: Vec<String> = args();
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

    gitstat::run(&path);
}

mod gitstat {
    use git2::{Repository,Commit,Oid};

    pub fn run(path: &Path) {
        let repo = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => fail!("{}", e),
        };

        let commit = match self::get_head_commit(&repo) {
            Some(commit) => commit,
            None => fail!("It seems like the repository is corrupt"),
        };

        let mut commits: Vec<Oid> = Vec::new();
        depth_first_search(&commit, &mut commits);
        println!("Total count of commits: {}", commits.len());
    }

    fn get_head_commit(repo: &Repository) -> Option<Commit> {
        repo.head().ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok())
    }

    fn depth_first_search(commit: &Commit, visited: &mut Vec<Oid>) {
        let id: Oid = commit.id();
        if !visited.contains(&id) {
            visited.push(id);
            for parent in commit.parents() {
                self::depth_first_search(&parent, visited);
            }
        }
    }
}
