use amaze::types::*;

pub fn main() {
    let mut maze = Maze::new(CoordXY { x: 0, y: 0 }, CoordXY { x: 7, y: 7 });
    maze.data[1].set_east(true);
    println!("{}", maze.data[0].into_bytes()[0]);
    println!("{}", maze.data[1].into_bytes()[0]);
}
