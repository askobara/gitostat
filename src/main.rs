#![feature(os)]
#![feature(core)]
#![feature(collections)]
#![feature(unicode)]
#![feature(str_char)]

extern crate git2;
extern crate chrono;
extern crate getopts;
extern crate unicode;

use getopts::Options;
use std::os;
use std::path::Path;

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args.tail()) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }

    let input = if !matches.free.is_empty() {
        println!("count of opts: {}", matches.free.len());
        matches.free[0].clone()
    } else {
        print_usage(program.as_slice(), opts);
        return;
    };

    let path = Path::new(input.as_slice());

    // println!("Total count of commits in '{}' is: {}", path.display(), stat.total);
    match gitstat::run(&path) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(brief.as_slice()));
}

pub struct Stat {
    total: usize,
}

mod gitstat {
    use git2::{Repository,Commit,Oid,Signature,Time,Error};
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;
    use std::sync::{Mutex, Arc};
    use std::path::Path;
    use chrono::datetime::DateTime;
    use chrono::naive::datetime::NaiveDateTime;
    use chrono::{Timelike, Datelike};
    use std::cmp;
    use Stat;

    pub fn run(path: &Path) -> Result<(), Error> {
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

    fn get_authors(repo: &Repository) -> Result<HashMap<String, usize>, Error> {
        let mut heatmap = [[0u32; 24]; 7];
        let mut authors: HashMap<String, usize> = HashMap::new();
        let mut revwalk = try!(repo.revwalk());

        revwalk.push_head();
        // let mutex = Mutex::new(repo);

        // mailmap::read_mailmap_file(path);
        for oid in revwalk {
            let commit = try!(repo.find_commit(oid));

            let uniq_name: String = get_uniq_name(&commit.author());
            let (weekday, hour) = get_heatmat_coords(&commit.time());

            heatmap[weekday as usize][hour as usize] += 1;
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
        for hours in heatmap.iter() {
            for count in hours.iter() {
                max = cmp::max(*count, max);
            }
        }

        let arts = ['.', '▪', '◾', '◼', '⬛'];

        for hours in heatmap.iter() {
            for count in hours.iter() {
                print!("{:4}", arts[(count % arts.len() as u32) as usize]);
                // print!("{:4}", count);
            }
            println!("");
        }

        Ok(authors)
    }

    fn get_uniq_name(author: &Signature) -> String {
        format!("{} <{}>", author.name().unwrap(), author.email().unwrap())
    }

    fn get_heatmat_coords(time: &Time) -> (u32, u32) {
        let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp(time.seconds(), 0);

        (timestamp.weekday().num_days_from_monday(), timestamp.hour())
    }

    /// Module `mailmap` which implement logic for parse .mailmap files
    mod mailmap {

        use std::env;
        use std::fs::File;
        use std::io::{BufReader, BufRead};
        use std::string::String;
        use std::path::Path;
        use unicode::str::UnicodeStr;

        pub fn read_mailmap_file(basedir: &Path) {
            // get full path to .mailmap
            let path = match env::current_dir() {
                Ok(path) => path.join(".mailmap"),
                Err(error) => panic!("{}", error),
            };

            // create BufReader instance for .mailmap file
            let mut reader = match File::open(&path) {
                Ok(file) => BufReader::new(file),
                Err(error) => panic!("{}", error),
            };

            for line in reader.lines() {
                match line {
                    Ok(line) => self::read_mailmap_line(line.as_slice()),
                    Err(x) => break
                };
            }
        }

        struct Author {
            name: String,
            email: String
        }

        struct MailmapLine {
            new_author: Author,
            old_author: Author
        }

        fn read_mailmap_line(line: &str) -> Option<MailmapLine> {
            if line.len() > 0 && line.char_at(0) != '#' {
                parse_mailmap_name_email(line);
            }

            None
        }

        fn parse_mailmap_name_email(line: &str) -> Option<Author> {
            let left = match line.find('<') {
                Some(i) => i,
                None => -1
            };

            let right = match line.find('>') {
                Some(i) => i,
                None => -1
            };

            if left == -1 || right == -1 {
                return None;
            }

            if left > right {
                return None;
            }

            if left+1 == right {
                return None;
            }

            Some(Author {
                name: String::from_str(UnicodeStr::trim(&line[..left])),
                email: String::from_str(UnicodeStr::trim(&line[left..right]))
            })

        }

        #[test]
        fn test_parse_name_email() {
            assert!(self::parse_mailmap_name_email("name <email>").is_some());
            assert!(self::parse_mailmap_name_email("name <>").is_none());
            assert!(self::parse_mailmap_name_email(">").is_none());
            assert!(self::parse_mailmap_name_email("<").is_none());
            assert!(self::parse_mailmap_name_email("><").is_none());
            assert!(self::parse_mailmap_name_email("").is_none());
            assert!(self::parse_mailmap_name_email("<email>").is_some());
        }

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
