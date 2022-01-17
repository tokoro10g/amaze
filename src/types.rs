#![forbid(unsafe_code)]

use modular_bitfield::prelude::*;

#[cfg(feature = "maze_32")]
const WIDTH: usize = 32;
#[cfg(not(feature = "maze_32"))]
const WIDTH: usize = 16;

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

#[derive(Debug, PartialEq, Eq)]
pub struct CoordXY {
    pub x: u8,
    pub y: u8,
}

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
#[derive(Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    CoordNotFound,
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
            Err(Error::CoordNotFound)
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
    pub const fn new(start: CoordXY, goal: CoordXY) -> Maze {
        Maze {
            start: start,
            goal: goal,
            data: [Cell::new(); WIDTH*WIDTH],
        }
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
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::West) == Err(Error::CoordNotFound));
        assert!(CoordXY { x: 0, y: 0 }.add(Direction::South) == Err(Error::CoordNotFound));
        assert!(
            CoordXY {
                x: (WIDTH - 1) as u8,
                y: 0
            }
            .add(Direction::East)
                == Err(Error::CoordNotFound)
        );
        assert!(
            CoordXY {
                x: 0,
                y: (WIDTH - 1) as u8
            }
            .add(Direction::North)
                == Err(Error::CoordNotFound)
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
