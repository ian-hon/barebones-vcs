use std::{collections::{HashMap, HashSet}, path::Path};

use similar::{ChangeTag, TextDiff};

use crate::content::{Content, Directory, File};

#[derive(Debug)]
pub struct Change {
    pub container_modifications: Vec<ContainerModification>,
    pub modifications: Vec<Modification>
}
impl Change {
    pub fn serialise_changes(self) -> String {
// + D "lorem/ipsum/dolor"
// + F "lorem/ipsum/dolor/earth.txt" "earth.txt"
// - D "lorem/sit"
// =
// | "lorem/ipsum/dolor/earth.txt"
// + 3 asdfsdf
// + 5 sfsdf
// - 7
// | "lorem/ipsum/saturn/txt"
// + 4 lsdfljs
        
        let mut result: Vec<String> = vec![];

        for c_m in self.container_modifications {
            result.push(
                match c_m {
                    ContainerModification::CreateDirectory(p, n) => {
                        format!("+ D {p:?} {n:?}")
                    },
                    ContainerModification::DeleteDirectory(p) => {
                        format!("- D {p:?}")
                    },
                    ContainerModification::CreateFile(p, n) => {
                        format!("+ F {p:?} {n:?}")
                    },
                    ContainerModification::DeleteFile(p) => {
                        format!("- F {p:?}")
                    }
                }
            );
        }

        result.push("=".to_string());

        let mut map = HashMap::new();
        for modification in &self.modifications {
            let path = match modification {
                Modification::Create(path, _, _, _) => path.clone(),
                Modification::Delete(path, _, _) => path.clone()
            };
            map.entry(path).or_insert(vec![]).push(modification.clone());
        }

        for (path, modifications) in map {
            result.push(format!("| {path:?}"));
            for m in modifications {
                result.push(
                    match m {
                        Modification::Create(_, _, line, content) => format!("+ {line} {content:?}"),
                        Modification::Delete(_, _, line) => format!("- {line}")
                    }
                )
            }
        }

        result.join("\n")
    }

    pub fn get_change(path: String, upstream_file: &File, current_file: &File) -> Vec<Modification> {
        // https://blog.jcoglan.com/2017/02/15/the-myers-diff-algorithm-part-2/
        // for our change algorithm, we will be using myers diff algorithm
        // basically a shortest distance problem, with downwards, rightwards and diagonal directions as movement choices
        // (note that diagonal movements do not contribute towards the distance)

        let upstream = upstream_file.content.clone();
        let current = current_file.content.clone();

        // TODO : compare hashes instead of files
        if upstream == current {
            return vec![];
        }

        let mut result = vec![];
        let diff = TextDiff::from_lines(&upstream, &current);

        for change in diff
            .iter_all_changes()
            .filter_map(|c| match c.tag() {
                ChangeTag::Equal => None,
                _ => Some(c)
            }
        ) {
            result.push(
                match change.tag() {
                    ChangeTag::Delete => Modification::Delete(
                        path.clone(),
                        current_file.name.clone(),
                        change.old_index().unwrap()
                    ),
                    ChangeTag::Insert => Modification::Create(
                        path.clone(),
                        current_file.name.clone(),
                        change.new_index().unwrap(),
                        change.to_string()
                    ),
                    _ => panic!()
                }
            )
        }

        result
    }

    pub fn get_change_all(upstream: &Directory, current: &Directory, path: &Path) -> Change {
        // assume that both current and previous have the same directory names
        // has to be bfs

        // initialise current state set
        let mut current_set = HashSet::new();
        let mut current_map = HashMap::new();
        for c in &current.content {
            match c {
                Content::Directory(d) => {
                    current_set.insert((d.name.clone(), false));
                    current_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    current_set.insert((f.name.clone(), true));
                    current_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // initialise upstream state set
        let mut upstream_set = HashSet::new();
        let mut upstream_map = HashMap::new();
        for c in &upstream.content {
            match c {
                Content::Directory(d) => {
                    upstream_set.insert((d.name.clone(), false));
                    upstream_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    upstream_set.insert((f.name.clone(), true));
                    upstream_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // use set differences to determine file and directory creation or deletion
        let deleted = upstream_set.difference(&current_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();
        let created = current_set.difference(&upstream_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();
        //

        // for all deleted files, log them
        // for all deleted directories, log them and do the same for all children
        let mut container_modifications = vec![];
        let mut modifications = vec![];
        for (dir_name, is_file) in deleted {
            if is_file {
                container_modifications.push(ContainerModification::DeleteFile(path.join(dir_name.clone()).to_string_lossy().to_string()));
            } else {
                let p = path.join(dir_name.clone());
                container_modifications.push(ContainerModification::DeleteDirectory(p.to_string_lossy().to_string()));
                // traverse all children, add them to result as well
                let mut changes = Change::get_change_all(
                    match upstream_map.get(&(dir_name, false)).unwrap() {
                        Content::Directory(deleted_d) => { deleted_d },
                        _ => panic!()
                    },
                    &Directory::new(),
                    &p
                );
                container_modifications.append(&mut changes.container_modifications);
                modifications.append(&mut changes.modifications);
            }
        }
        //

        // for all created files, log them
        // for all created directories, log them and do the same for all children
        for (dir_name, is_file) in created {
            if is_file {
                let p = path.join(dir_name.clone()).to_string_lossy().to_string();
                container_modifications.push(ContainerModification::CreateFile(p.clone(), dir_name.clone()));
                // Modification::Create here
                modifications.append(&mut Change::get_change(p, &File::new(), match current_map.get(&(dir_name, true)).unwrap() {
                    Content::File(f) => { f },
                    _ => panic!()
                }))
            } else {
                let p = path.join(dir_name.clone());
                container_modifications.push(ContainerModification::CreateDirectory(p.to_string_lossy().to_string(), dir_name.clone()));

                let mut changes = Change::get_change_all(
                    &Directory::new(),
                    match current_map.get(&(dir_name, false)).unwrap() {
                        Content::Directory(d) => d,
                        _ => panic!()
                    },
                    &p
                );
                container_modifications.append(&mut changes.container_modifications);
                modifications.append(&mut changes.modifications);
            }
        }

        for content in &current.content {
            match content {
                Content::Directory(directory) => {
                    // get the matching upstream directory
                    // if it doesnt exist, that means the content is new and can be ignored
                    // we ignore it because we have already logged it in the section above
                    let p = path.join(directory.name.clone());
                    let upstream_directory = match upstream_map.get(&(directory.name.clone(), false)) {
                        Some(u) => {
                            match u {
                                Content::Directory(u_d) => { u_d },
                                _ => panic!()
                            }
                        },
                        _ => { continue; }
                    };
                    //

                    let mut changes = Change::get_change_all(
                        upstream_directory,
                        directory,
                        &p
                    );
                    container_modifications.append(&mut changes.container_modifications);
                    modifications.append(&mut changes.modifications);
                },
                Content::File(f) => {
                    let upstream_file = match upstream_map.get(&(f.name.clone(), true)) 
                    {
                        Some(c) => match c {
                            Content::File(f) => f,
                            _ => panic!()
                        },
                        None => { continue; }
                    };

                    modifications.append(&mut Change::get_change(path.join(f.name.clone()).to_string_lossy().to_string(), &upstream_file, &f));
                }
            }
        }

        Change {
            container_modifications,
            modifications
        }
    }
}

#[derive(Debug, Clone)]
pub enum Modification {
    // creation/deletion of lines in files
    Create(
        String, // path
        String, // file name
        usize, // line
        String // text
    ),
    Delete(
        String, // path
        String, // file name
        usize // line
    )
}

#[derive(Debug)]
pub enum ContainerModification {
    // creation/deletion of files & folders
    // TODO : change so only path needed
    CreateDirectory(
        String, // path
        String // name
    ),
    DeleteDirectory(
        String // path
    ),

    CreateFile(
        String, // path
        String // name
    ),
    DeleteFile(
        String, // path
    )
}