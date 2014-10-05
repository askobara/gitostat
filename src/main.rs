extern crate getopts;
extern crate git2;

use std::os;
use std::io::fs::PathExtensions;
use getopts::{optflag,getopts};
use git2::Repository;

fn is_git_repo(path: &Path) -> bool {
    let abs_path: Path = os::make_absolute(path).join(".git");

    return abs_path.exists() && abs_path.is_dir();
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
        println!("git repo");
        let repo = match Repository::open(&path) {
            Ok(repo) => repo,
            Err(e) => fail!("failed to open `{}`: {}", path.display(), e),
        };

        let mut branches_iter = match repo.branches(None) {
            Ok(b) => { b }
            Err(e) => {
                fail!(e.to_string());
                return;
            }
        };

        for branch in branches_iter {
            match branch {
                (b, t) => {
                    println!("{} {}", b.name(), t);
                }
            }
        }
    }
}

