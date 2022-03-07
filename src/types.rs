#![forbid(unsafe_code)]

use core::fmt;
use modular_bitfield::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "maze_8x8")] {
        const WIDTH: usize = 8;
    } else if #[cfg(feature = "maze_16x16")] {
        const WIDTH: usize = 16;
    } else if #[cfg(feature = "maze_32x32")] {
        const WIDTH: usize = 32;
    } else {
        compile_error!("Select one of features: maze_{8x8, 16x16, 32x32}");
    }
}

#[bitfield]
#[derive(Copy, Clone)]
pub struct Cell {
    pub north: bool,
    pub east: bool,
    pub west: bool,
    pub south: bool,
    pub chk_north: bool,
    pub chk_east: bool,
    pub chk_west: bool,
    pub chk_south: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CoordXY {
    pub x: u8,
    pub y: u8,
}

#[non_exhaustive]
pub struct Maze {
    pub start: CoordXY,
    pub goal: CoordXY,
    pub data: [Cell; WIDTH * WIDTH],
}

#[derive(Debug, PartialEq, Eq)]
pub struct VectorXY {
    pub x: i8,
    pub y: i8,
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfRange,
}

impl CoordXY {
    #[inline]
    pub const fn add(self, dir: Direction) -> Result<CoordXY, Error> {
        let d = dir.to_vector_xy();
        let new_x = self.x as i16 + d.x as i16;
        let new_y = self.y as i16 + d.y as i16;
        if new_x >= 0 && new_y >= 0 && (new_x as usize) < WIDTH && (new_y as usize) < WIDTH {
            Ok(CoordXY {
                x: new_x as u8,
                y: new_y as u8,
            })
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl Direction {
    #[inline]
    pub const fn to_vector_xy(self) -> VectorXY {
        use Direction::*;
        match self {
            North => VectorXY { x: 0, y: 1 },
            East => VectorXY { x: 1, y: 0 },
            South => VectorXY { x: 0, y: -1 },
            West => VectorXY { x: -1, y: 0 },
        }
    }
}

impl Maze {
    pub fn new(start: CoordXY, goal: CoordXY) -> Maze {
        let mut data = [Cell::new(); WIDTH * WIDTH];
        for x in 0..WIDTH {
            data[x].set_south(true);
            data[x + WIDTH * (WIDTH - 1)].set_north(true);
        }
        for y in 0..WIDTH {
            data[y * WIDTH].set_west(true);
            data[WIDTH - 1 + y * WIDTH].set_east(true);
        }
        Maze { start, goal, data }
    }

    pub fn load(maze_str: &str) -> Maze {
        // TODO(tokoro10g): Find start and goal
        let mut maze = Maze::new(CoordXY { x: 0, y: 0 }, CoordXY { x: 7, y: 7 });
        let mut width = 0;
        for w in [32, 16, 9, 8] {
            let nominal_len = (4 * w + 2) * (2 * w + 1);
            if maze_str.len() / nominal_len == 1 {
                width = w;
                break;
            }
        }
        if width > WIDTH {
            panic!("Loaded data is too large");
        }
        let mut coord = CoordXY {
            x: 0,
            y: (width - 1) as u8,
        };
        for (line_no, s) in maze_str.split('\n').enumerate() {
            coord.y = (width - 1 - line_no / 2) as u8;
            if line_no % 2 == 0 {
                // Check for walls in north or south
                for x in 0..width {
                    coord.x = x as u8;
                    if s.as_bytes()[2 + 4 * x] == b'-' {
                        maze.modify_data(coord, Direction::North, true);
                    }
                }
            } else {
                // Check for walls in west or east
                for x in 0..width {
                    coord.x = x as u8;
                    if s.as_bytes()[4 * x] == b'|' {
                        maze.modify_data(coord, Direction::West, true);
                    }
                    if s.as_bytes()[4 * x + 2] == b'S' {
                        maze.start = coord;
                    } else if s.as_bytes()[4 * x + 2] == b'G' {
                        maze.goal = coord;
                    }
                    if s.as_bytes()[4 * x + 4] == b'|' {
                        maze.modify_data(coord, Direction::East, true);
                    }
                }
                if coord.y == 0 {
                    break;
                }
            }
        }
        maze
    }

    pub fn get_mutable_cell(&mut self, coord: CoordXY) -> &mut Cell {
        &mut self.data[coord.x as usize + coord.y as usize * WIDTH]
    }

    pub fn modify_data(&mut self, coord: CoordXY, direction: Direction, value: bool) {
        match direction {
            Direction::North => self.get_mutable_cell(coord).set_north(value),
            Direction::East => self.get_mutable_cell(coord).set_east(value),
            Direction::South => self.get_mutable_cell(coord).set_south(value),
            Direction::West => self.get_mutable_cell(coord).set_west(value),
        }
        if let Ok(next_coord) = coord.add(direction) {
            match direction {
                Direction::North => self.get_mutable_cell(next_coord).set_south(value),
                Direction::East => self.get_mutable_cell(next_coord).set_west(value),
                Direction::South => self.get_mutable_cell(next_coord).set_north(value),
                Direction::West => self.get_mutable_cell(next_coord).set_east(value),
            }
        }
    }
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..WIDTH).rev() {
            for x in 0..WIDTH {
                let cell = self.data[x + y * WIDTH];
                write!(f, "+{}", if cell.north() { "---" } else { "   " }).unwrap();
            }
            writeln!(f, "+").unwrap();
            for x in 0..WIDTH {
                let cell = self.data[x + y * WIDTH];
                let coord = CoordXY {
                    x: x as u8,
                    y: y as u8,
                };
                let mut cell_mark = " ";
                if self.start == coord {
                    cell_mark = "S";
                } else if self.goal == coord {
                    cell_mark = "G";
                }
                write!(f, "{} {} ", if cell.west() { "|" } else { " " }, cell_mark).unwrap();
            }
            writeln!(f, "|").unwrap();
        }
        for _ in 0..WIDTH {
            write!(f, "+---").unwrap();
        }
        writeln!(f, "+").unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_xy_add() {
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::North) == Ok(CoordXY { x: 0, y: 1 }));
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::East) == Ok(CoordXY { x: 1, y: 0 }));
    }

    #[test]
    fn coord_xy_add_overflow() {
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::West) == Err(Error::OutOfRange));
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::South) == Err(Error::OutOfRange));
        assert!(
            CoordXY {
                x: (WIDTH - 1) as u8,
                y: 0
            }
            .add(Direction::East)
                == Err(Error::OutOfRange)
        );
        assert!(
            CoordXY {
                x: 0,
                y: (WIDTH - 1) as u8
            }
            .add(Direction::North)
                == Err(Error::OutOfRange)
        );
    }

    #[test]
    fn direction_to_vector_xy() {
        assert!(Direction::North.to_vector_xy() == VectorXY { x: 0, y: 1 });
        assert!(Direction::East.to_vector_xy() == VectorXY { x: 1, y: 0 });
        assert!(Direction::South.to_vector_xy() == VectorXY { x: 0, y: -1 });
        assert!(Direction::West.to_vector_xy() == VectorXY { x: -1, y: 0 });
    }
}
