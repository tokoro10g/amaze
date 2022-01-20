use amaze::types::*;
use std::fs;

pub fn main() {
    let data = fs::read_to_string("maze.txt").expect("Unable to read file");
    let maze = Maze::load(&data);
    println!("{}", maze);
}
