use crate::state::State;

pub fn add(state: State, args: Vec<String>) {

}

pub fn commit(state: State, args: Vec<String>) {
    // everything after the first line will be generated by Change::serialise_change
r#"= {unix timestamp of commit} "{message}" "{description}"
+ D "lorem/ipsum/dolor"
+ F "lorem/ipsum/dolor/earth.txt" "earth.txt"
- D "lorem/sit"
=
| "lorem/ipsum/dolor/earth.txt"
+ 3 asdfsdf
+ 5 sfsdf
- 7
| "lorem/ipsum/saturn/txt"
+ 4 lsdfljs"#;
}

pub fn push(state: State, args: Vec<String>) {

}

pub fn pull(state: State, args: Vec<String>) {

}

pub fn fetch(state: State, args: Vec<String>) {

}

pub fn cherry(state: State, args: Vec<String>) {
    
}

pub fn rollback(state: State, args: Vec<String>) {

}