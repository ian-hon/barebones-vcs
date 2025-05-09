use std::fs;
use std::path::Path;
use std::{collections::HashMap, env};

mod error;
mod ignore;
mod state;
mod utils;

mod relic;
mod commit;
mod branch;
mod stash;

mod content;
mod change;

use relic::Relic;
use change::Change;
use content::File;
use ignore::IgnoreSet;
use utils::generate_tree;

use crate::commit::{add, commit, push, pull, fetch, cherry, rollback};
use crate::branch::branch;
use crate::stash::{stash, restore};
use crate::state::State;

// add
// commit {message}
// push
// pull
// fetch
// branch {name}
//      will change to that branch
//      if branch doesnt exist, create
//      ask to create stash (if changes present)
// stash {name|optional}
//      stashes are bound to a branch
//      optional to have a name
// restore
//      select stash to restore
// rollback
//      resets to current head
// cherry {commit hash}

pub fn init(_: State, _: Vec<String>) {

}


pub fn help(_: State, _: Vec<String>) {
    println!(r#"This is the Relic Version Control System."#);
}

pub fn about(_: State, _: Vec<String>) {
    println!(r#"This is the Relic Version Control System.

The best way to learn is to stupidly and
blindly reinvent the wheel.

Relic is a simple
hobby project, because remaking Git sounded
fun and interesting.

Most common features like adding,
committing, pushing and pulling, are
implemented."#);
}

fn main() {
    // for change in Change::get_change(
    //     "".to_string(),
    //     &File { name: "".to_string(), content: fs::read_to_string("./lorem/mars").unwrap() },
    //     &File { name: "".to_string(), content: fs::read_to_string("./lorem/earth").unwrap() })
    // {
    //     println!("{change:?}");
    // }
    // return;

    // println!("{:?}", Change::get_change_container(
    //     &Directory {
    //         name: "test".to_string(),
    //         content: vec![
    //             Content::Directory(Directory {
    //                 name: "dolor".to_string(),
    //                 content: vec![]
    //             }),
    //             Content::Directory(Directory {
    //                 name: "sit".to_string(),
    //                 content: vec![
    //                     Content::Directory(Directory {
    //                         name: "new_dir".to_string(),
    //                         content: vec![
    //                             Content::File(File {
    //                                 name: "smaller.txt".to_string(),
    //                                 content: "".to_string()
    //                             }),
    //                         ]
    //                     }),
    //                 ]
    //             }),
    //             Content::File(File {
    //                 name: "test.txt".to_string(),
    //                 content: "".to_string()
    //             })
    //         ]
    //     },
    //    &Directory {
    //         name: "test".to_string(),
    //         content: vec![
    //             Content::Directory(Directory {
    //                 name: "sit".to_string(),
    //                 content: vec![
    //                     Content::Directory(Directory {
    //                         name: "new_dir".to_string(),
    //                         content: vec![
    //                             Content::Directory(Directory {
    //                                 name: "small".to_string(),
    //                                 content: vec![]
    //                             }),
    //                             Content::File(File {
    //                                 name: "smaller.txt".to_string(),
    //                                 content: "".to_string()
    //                             }),
    //                         ]
    //                     }),
    //                 ]
    //             }),
    //             Content::File(File {
    //                 name: "test.txt".to_string(),
    //                 content: "".to_string()
    //             }),
    //         ]
    //     },
    //     Path::new("here")
    // ));

    // build all commands
    type CommandType = fn(State, Vec<String>);
    let commands: HashMap<String, CommandType> = HashMap::from_iter::<Vec<(String, CommandType)>>(vec![
        ("add".to_string(), add),
        ("commit".to_string(), commit),
        ("push".to_string(), push),
        ("pull".to_string(), pull),
        ("fetch".to_string(), fetch),
        ("branch".to_string(), branch),
        ("stash".to_string(), stash),
        ("restore".to_string(), restore),
        ("rollback".to_string(), rollback),
        ("cherry".to_string(), cherry),
        ("help".to_string(), help),
        ("about".to_string(), about),

        ("tree".to_string(), |s, _| {
            println!("{}", generate_tree(&s));
        }),

        ("init".to_string(), init),

        ("update".to_string(), |s, _| {
            let _ = fs::write("./.relic/upstream", s.serialise_state());
        })
    ]);
    //
    
    // collect all arguments
    let arguments = env::args().collect::<Vec<String>>();
    if arguments.len() <= 1 {
        help(State::empty(), vec![]);
        return;
    }
    //

    // get current path
    // (used to be more complicated than this, but keeping it as a relative path just makes more sense now)
    let path = Path::new(".");
    //

    // get ignorance set
    let ignore_set = IgnoreSet::create(fs::read_to_string(path.join(".relic_ignore")).unwrap_or("".to_string()));
    //

    // get upstream
    // return error if not found
    let upstream_state = match Relic::load(&path) {
        Some(u) => { u },
        None => {
            println!("Upstream does not exist, consider running 'relic init' instead.");
            Relic::empty()
        }
    };
    //

    match State::create(".".to_string(), &ignore_set) {
        Ok(s) => {
            println!("{}", Change::get_change_all(&upstream_state.upstream.current, &s.current, &path).serialise_changes());

            // println!("Upstream :\n{}", generate_tree(&upstream_state.upstream));
            // println!("Current :\n{}", generate_tree(&s));
            // fs::write("./.relic/upstream", s.serialise_state());

            match commands.get(&arguments[1]) {
                Some(c) => {
                    c(s, arguments[2..arguments.len()].to_vec());
                },
                None => {
                    println!("Command not found.");
                    help(State::empty(), vec![]);
                }
            }
        },
        Err(e) => {
            println!("{e:?} error encountered.")
        }
    }
}
