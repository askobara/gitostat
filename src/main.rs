extern crate getopts;
extern crate git2;

use std::os::{args};
use getopts::{optflag,getopts};

#[cfg(not(test))]
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

    let stat = gitstat::run(&path).ok().expect("All gonna bad");
    println!("Total count of commits in '{}' is: {}", path.display(), stat.total);
}

struct Stat {
    total: uint,
}

mod gitstat {
    use git2::{Repository,Commit,Oid};
    use Stat;

    pub fn run(path: &Path) -> Result<Stat, &str> {
        let repo = Repository::open(path).ok().expect("Not valid git repository");

        let commit = match self::get_head_commit(&repo) {
            Some(commit) => commit,
            None => fail!("It seems like the repository is corrupt"),
        };

        let mut commits: Vec<Oid> = Vec::new();
        depth_first_search(&commit, &mut commits);

        Ok(Stat { total: commits.len() })
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

#[cfg(test)]
mod tests {
    use std::io::TempDir;
    use git2::Repository;
    use gitstat::run;

    fn repo_init() -> (TempDir, Repository) {
        let td = TempDir::new("test").unwrap();
        let repo = Repository::init(td.path()).unwrap();
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "name").unwrap();
            config.set_str("user.email", "email").unwrap();
            let mut index = repo.index().unwrap();
            let id = index.write_tree().unwrap();

            let tree = repo.find_tree(id).unwrap();
            let sig = repo.signature().unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, []).unwrap();
        }
        (td, repo)
    }

    #[test]
    fn smoke_run() {
        let (_td, repo) = self::repo_init();
        let path = repo.path();
        let result = run(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().total, 1);
    }
}
