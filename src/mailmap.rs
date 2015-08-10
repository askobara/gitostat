use std::path::Path;
use std::{fmt, default};
use std::io::{BufReader,BufRead};
use std::fs::File;
use std::collections::HashMap;
use std::string;
use git2;

macro_rules! ostring {
    ($e:expr) => (match $e {
        Some(s) => Some(String::from(s)),
        None => None
    })
}

struct Author {
    name: Option<String>,
    email: Option<String>,
    namemap: HashMap<String, Author>
}

impl Author {
    pub fn new(name: Option<String>, email: Option<String>) -> Author {
        Author {
            name: name,
            email: email,
            namemap: HashMap::new()
        }
    }

    pub fn set_name(&mut self, name: Option<String>) -> &mut Author {
        self.name = name;
        self
    }

    pub fn set_email(&mut self, email: Option<String>) -> &mut Author {
        self.email = email;
        self
    }

    pub fn namemap_insert(&mut self, name: String, author: Author) -> &mut Author {
        self.namemap.insert(name, author);
        self
    }
}

impl default::Default for Author {
    fn default() -> Author {
        Author {
            name: None,
            email: None,
            namemap: HashMap::new()
        }
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.name, self.email)
    }
}

impl fmt::Debug for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} <{:?}> {:?}", self.name, self.email, self.namemap)
    }
}

pub struct Mailmap {
    items: HashMap<String, Author>
}

impl Mailmap {

    pub fn new(path: &Path) -> Option<Mailmap> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None
        };

        let reader = BufReader::new(file);

        // for more help with this regex see https://www.debuggex.com/r/eF5E6HQm4aAhXEtN
        let re = regex!(r"^((?P<new_name>.+?)\s+)??<\s*(?P<new_email>.+?)\s*>((\s+(?P<old_name>.+?))??\s+<\s*(?P<old_email>.+?)\s*>)?");

        let mut authors: HashMap<String, Author> = HashMap::new();

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => continue
            };

            // no comments
            if line.chars().nth(0) == Some('#') { continue; }

            if let Some(caps) = re.captures(&line[..]) {
                let mut old_email = ostring!(caps.name("old_email"));
                let old_name  = ostring!(caps.name("old_name"));
                let mut new_email = ostring!(caps.name("new_email"));
                let new_name  = ostring!(caps.name("new_name"));

                if old_email.is_none() {
                    old_email = new_email.clone();
                    new_email = None;
                }

                let mut me = authors.entry(old_email.unwrap()).or_insert(Author::default());

                if let Some(name) = old_name {
                    me.namemap_insert(name, Author::new(new_name, new_email));
                } else {
                    me.set_name(new_name).set_email(new_email);
                }
            }
        }

        Some(Mailmap {
            items: authors
        })
    }

    pub fn map_user(&self, signature: &git2::Signature) -> Result<String, string::FromUtf8Error> {
        let name =  ostring!(signature.name()).expect("Valid utf8 string");
        let email = ostring!(signature.email()).expect("Valid utf8 string");
        let author = match self.items.get(&email) {
            Some(item) => if item.namemap.is_empty() { Some(item) } else { item.namemap.get(&name) },
            None => None
        };

        Ok(author.map_or_else(|| format!("{}", signature), |item| format!("{} <{}>", item.name.clone().unwrap_or(name), item.email.clone().unwrap_or(email))))
    }

}

