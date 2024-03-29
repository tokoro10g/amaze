#![forbid(unsafe_code)]

use core::{fmt, ops::Add, ops::Sub};
pub use heapless::Vec;
use modular_bitfield::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "maze_8x8")] {
        pub const WIDTH: usize = 8;
    } else if #[cfg(feature = "maze_16x16")] {
        pub const WIDTH: usize = 16;
    } else if #[cfg(feature = "maze_32x32")] {
        pub const WIDTH: usize = 32;
    } else {
        compile_error!("Select one of features: maze_{8x8, 16x16, 32x32}");
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    OutOfRange,
    InvalidLocation,
    InvalidDirection,
    InvalidVector,
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
impl Direction {
    #[inline]
    pub fn inverted(&self) -> Self {
        use Direction::*;
        match *self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}
impl TryFrom<VectorXY> for Direction {
    type Error = Error;
    #[inline]
    fn try_from(value: VectorXY) -> Result<Self, Error> {
        use Direction::*;
        match value {
            VectorXY { x: 0, y: 1 } => Ok(North),
            VectorXY { x: 1, y: 0 } => Ok(East),
            VectorXY { x: 0, y: -1 } => Ok(South),
            VectorXY { x: -1, y: 0 } => Ok(West),
            _ => Err(Error::InvalidVector),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub struct Coord1D {
    value: u8,
}
impl Coord1D {
    pub const MAX: u8 = WIDTH as u8 - 1;
    cfg_if::cfg_if! {
        if #[cfg(feature = "debug")]{
            #[inline]
            pub fn new(value: u8) -> Result<Self, Error> {
                if value > Self::MAX {
                    Err(Error::OutOfRange)
                } else {
                    Ok(Self { value })
                }
            }
        } else {
            #[inline]
            pub fn new(value: u8) -> Result<Self, Error> {
                Ok(Self { value })
            }
        }
    }
    #[inline]
    pub fn value(&self) -> u8 {
        self.value
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CoordXY {
    x: Coord1D,
    y: Coord1D,
}
impl CoordXY {
    #[inline]
    pub fn new(x: u8, y: u8) -> Result<Self, Error> {
        if let (Ok(x), Ok(y)) = (Coord1D::new(x), Coord1D::new(y)) {
            Ok(Self { x, y })
        } else {
            Err(Error::OutOfRange)
        }
    }
    #[inline]
    pub fn x(&self) -> Coord1D {
        self.x
    }
    #[inline]
    pub fn y(&self) -> Coord1D {
        self.y
    }
}
impl Add<VectorXY> for CoordXY {
    type Output = Result<CoordXY, Error>;
    fn add(self, rhs: VectorXY) -> Self::Output {
        let new_x = self.x.value as i16 + rhs.x as i16;
        let new_y = self.y.value as i16 + rhs.y as i16;
        if new_x >= 0
            && new_y >= 0
            && (new_x as u8) <= Coord1D::MAX
            && (new_y as u8) <= Coord1D::MAX
        {
            Ok(CoordXY::new(new_x as u8, new_y as u8).unwrap())
        } else {
            Err(Error::OutOfRange)
        }
    }
}
impl Sub<CoordXY> for CoordXY {
    type Output = VectorXY;
    fn sub(self, rhs: CoordXY) -> Self::Output {
        VectorXY {
            x: self.x.value as i8 - rhs.x.value as i8,
            y: self.y.value as i8 - rhs.y.value as i8,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VectorXY {
    pub x: i8,
    pub y: i8,
}
impl From<Direction> for VectorXY {
    #[inline]
    fn from(value: Direction) -> Self {
        use Direction::*;
        match value {
            North => VectorXY { x: 0, y: 1 },
            East => VectorXY { x: 1, y: 0 },
            South => VectorXY { x: 0, y: -1 },
            West => VectorXY { x: -1, y: 0 },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CellLocalLocation {
    Center,
    North,
    East,
    South,
    West,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AgentState {
    pub location: CoordXY,
    pub local_location: CellLocalLocation,
    pub heading_vector: VectorXY,
}

#[bitfield]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
    pub check_north: bool,
    pub check_east: bool,
    pub check_south: bool,
    pub check_west: bool,
}
impl Cell {
    pub fn state_by_direction(&self, direction: Direction) -> bool {
        use Direction::*;
        match direction {
            North => self.north(),
            East => self.east(),
            South => self.south(),
            West => self.west(),
        }
    }
    pub fn check_by_direction(&self, direction: Direction) -> bool {
        use Direction::*;
        match direction {
            North => self.check_north(),
            East => self.check_east(),
            South => self.check_south(),
            West => self.check_west(),
        }
    }
    pub fn set_state_by_direction(&mut self, direction: Direction, value: bool) {
        use Direction::*;
        match direction {
            North => self.set_north(value),
            East => self.set_east(value),
            South => self.set_south(value),
            West => self.set_west(value),
        }
    }
    pub fn set_check_by_direction(&mut self, direction: Direction, value: bool) {
        use Direction::*;
        match direction {
            North => self.set_check_north(value),
            East => self.set_check_east(value),
            South => self.set_check_south(value),
            West => self.set_check_west(value),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Maze {
    pub start: CoordXY,
    pub goal: CoordXY,
    pub data: [Cell; WIDTH * WIDTH],
}
impl Maze {
    pub fn new(start: CoordXY, goal: CoordXY) -> Self {
        let mut data = [Cell::new(); WIDTH * WIDTH];
        for x in 0..WIDTH {
            data[x].set_south(true);
            data[x + WIDTH * (WIDTH - 1)].set_north(true);
        }
        for y in 0..WIDTH {
            data[y * WIDTH].set_west(true);
            data[WIDTH - 1 + y * WIDTH].set_east(true);
        }
        Self { start, goal, data }
    }
    pub fn load_from_str(maze_str: &str) -> Self {
        let mut maze = Self::new(CoordXY::new(0, 0).unwrap(), CoordXY::new(7, 7).unwrap());
        let mut width = 0;
        // TODO: Support arbitrary size
        for w in [32, 16, 9, 8, 4] {
            let nominal_len = (4 * w + 2) * (2 * w + 1);
            if maze_str.len() / nominal_len == 1 {
                width = w;
                break;
            }
        }
        if (width > WIDTH) || (width == 0) {
            panic!("Loaded data has invalid size {}", width);
        }
        let mut coord = CoordXY::new(0, (width - 1) as u8).unwrap();
        for (line_no, s) in maze_str.split('\n').enumerate() {
            coord.y = Coord1D::new((width - 1 - line_no / 2) as u8).unwrap();
            if line_no % 2 == 0 {
                // Check for walls in north or south
                for x in 0..width {
                    coord.x = Coord1D::new(x as u8).unwrap();
                    if s.as_bytes()[2 + 4 * x] == b'-' {
                        maze.set_cell_state(coord, Direction::North, true);
                    }
                }
            } else {
                // Check for walls in west or east
                for x in 0..width {
                    coord.x = Coord1D::new(x as u8).unwrap();
                    if s.as_bytes()[4 * x] == b'|' {
                        maze.set_cell_state(coord, Direction::West, true);
                    }
                    if s.as_bytes()[4 * x + 2] == b'S' {
                        maze.start = coord;
                    } else if s.as_bytes()[4 * x + 2] == b'G' {
                        maze.goal = coord;
                    }
                    if s.as_bytes()[4 * x + 4] == b'|' {
                        maze.set_cell_state(coord, Direction::East, true);
                    }
                }
                if coord.y.value == 0 {
                    break;
                }
            }
        }
        maze
    }
    #[inline]
    pub fn cell_by_x_y(&self, x: Coord1D, y: Coord1D) -> Cell {
        // NOTE: it is ensured that `x` and `y` are within the range [0, WIDTH).
        self.data[x.value as usize + y.value as usize * WIDTH]
    }
    #[inline]
    pub fn cell(&self, coord: CoordXY) -> Cell {
        self.cell_by_x_y(coord.x, coord.y)
    }
    #[inline]
    pub fn mutable_cell_by_x_y(&mut self, x: Coord1D, y: Coord1D) -> &mut Cell {
        // NOTE: it is ensured that `x` and `y` are within the range [0, WIDTH).
        &mut self.data[x.value as usize + y.value as usize * WIDTH]
    }
    #[inline]
    pub fn mutable_cell(&mut self, coord: CoordXY) -> &mut Cell {
        self.mutable_cell_by_x_y(coord.x, coord.y)
    }
    pub fn set_cell_state(&mut self, coord: CoordXY, direction: Direction, state: bool) {
        self.mutable_cell(coord)
            .set_state_by_direction(direction, state);
        if let Ok(next_coord) = coord + direction.into() {
            self.mutable_cell(next_coord)
                .set_state_by_direction(direction.inverted(), state);
        }
    }
    pub fn set_cell_check(&mut self, coord: CoordXY, direction: Direction, state: bool) {
        self.mutable_cell(coord)
            .set_check_by_direction(direction, state);
        if let Ok(next_coord) = coord + direction.into() {
            self.mutable_cell(next_coord)
                .set_check_by_direction(direction.inverted(), state);
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
                let coord = CoordXY::new(x as u8, y as u8).unwrap();
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

    const MAZE_STR: &str = "\
        +   +   +   +   +\n\
        |                \n\
        +   +   +   +   +\n\
        |                \n\
        +   +   +   +   +\n\
        |                \n\
        +---+---+   +   +\n\
        |       |        \n\
        +---+---+---+---+\n";

    #[test]
    fn direction_inverted() {
        assert_eq!(Direction::North.inverted(), Direction::South);
        assert_eq!(Direction::East.inverted(), Direction::West);
        assert_eq!(Direction::South.inverted(), Direction::North);
        assert_eq!(Direction::West.inverted(), Direction::East);
        assert_eq!(Direction::North.inverted().inverted(), Direction::North);
        assert_eq!(Direction::East.inverted().inverted(), Direction::East);
    }
    #[test]
    fn direction_from_vector_xy() {
        assert_eq!(
            Direction::North,
            VectorXY { x: 0, y: 1 }.try_into().unwrap()
        );
        assert_eq!(Direction::East, VectorXY { x: 1, y: 0 }.try_into().unwrap());
        assert_eq!(
            Direction::South,
            VectorXY { x: 0, y: -1 }.try_into().unwrap()
        );
        assert_eq!(
            Direction::West,
            VectorXY { x: -1, y: 0 }.try_into().unwrap()
        );
    }
    #[test]
    fn direction_from_vector_xy_invalid_vector() {
        let direction: Result<Direction, Error> = VectorXY { x: 2, y: 3 }.try_into();
        assert!(direction.is_err());
        assert_eq!(direction.err(), Some(Error::InvalidVector));
    }
    #[test]
    fn coord_1d() {
        assert_eq!(Coord1D::new(0).unwrap().value(), 0);
    }
    #[test]
    fn coord_1d_out_of_range() {
        assert_eq!(Coord1D::new(255), Err(Error::OutOfRange));
    }
    #[test]
    fn coord_xy_new() {
        let c = CoordXY::new(0, 1).unwrap();
        assert_eq!(c.x(), Coord1D::new(0).unwrap());
        assert_eq!(c.y(), Coord1D::new(1).unwrap());
    }
    #[test]
    fn coord_xy_new_out_of_range() {
        assert_eq!(CoordXY::new(0, 255), Err(Error::OutOfRange));
    }
    #[test]
    fn coord_xy_add() {
        assert_eq!(
            CoordXY::new(0, 0).unwrap() + Direction::North.into(),
            Ok(CoordXY::new(0, 1).unwrap())
        );
        assert_eq!(
            CoordXY::new(0, 0).unwrap() + Direction::East.into(),
            Ok(CoordXY::new(1, 0).unwrap())
        );
    }
    #[test]
    fn coord_xy_add_out_of_range() {
        assert_eq!(
            CoordXY::new(0, 0).unwrap() + Direction::West.into(),
            Err(Error::OutOfRange)
        );
        assert_eq!(
            CoordXY::new(0, 0).unwrap() + Direction::South.into(),
            Err(Error::OutOfRange)
        );
        assert_eq!(
            CoordXY::new(Coord1D::MAX, 0).unwrap() + Direction::East.into(),
            Err(Error::OutOfRange)
        );
        assert_eq!(
            CoordXY::new(0, Coord1D::MAX).unwrap() + Direction::North.into(),
            Err(Error::OutOfRange)
        );
    }
    #[test]
    fn coord_xy_sub() {
        assert_eq!(
            CoordXY::new(2, 1).unwrap() - CoordXY::new(1, 0).unwrap(),
            VectorXY { x: 1, y: 1 }
        );
        assert_eq!(
            CoordXY::new(1, 0).unwrap() - CoordXY::new(1, 2).unwrap(),
            VectorXY { x: 0, y: -2 }
        );
    }
    #[test]
    fn direction_into_vector_xy() {
        assert_eq!(VectorXY { x: 0, y: 1 }, Direction::North.into());
        assert_eq!(VectorXY { x: 1, y: 0 }, Direction::East.into());
        assert_eq!(VectorXY { x: 0, y: -1 }, Direction::South.into());
        assert_eq!(VectorXY { x: -1, y: 0 }, Direction::West.into());
    }
    #[test]
    fn cell_state_by_direction() {
        let mut cell = Cell::new();
        cell.set_east(true);
        assert!(cell.state_by_direction(Direction::East));
        assert!(!cell.state_by_direction(Direction::North));
    }
    #[test]
    fn cell_check_by_direction() {
        let mut cell = Cell::new();
        cell.set_check_east(true);
        assert!(cell.check_by_direction(Direction::East));
        assert!(!cell.check_by_direction(Direction::North));
    }
    #[test]
    fn cell_set_state_by_direction() {
        let mut cell = Cell::new();
        cell.set_state_by_direction(Direction::East, true);
        assert!(cell.east());
        cell.set_state_by_direction(Direction::East, false);
        assert!(!cell.east());
    }
    #[test]
    fn cell_set_check_by_direction() {
        let mut cell = Cell::new();
        cell.set_check_by_direction(Direction::East, true);
        assert!(cell.check_east());
        cell.set_check_by_direction(Direction::East, false);
        assert!(!cell.check_east());
    }
    #[test]
    fn maze_new() {
        let maze = Maze::new(CoordXY::new(0, 0).unwrap(), CoordXY::new(1, 1).unwrap());
        assert_eq!(maze.start, CoordXY::new(0, 0).unwrap());
        assert_eq!(maze.goal, CoordXY::new(1, 1).unwrap());
        assert!(!maze.data[0].north());
        assert!(!maze.data[0].east());
        assert!(maze.data[0].south());
        assert!(maze.data[0].west());
    }
    #[test]
    fn maze_cell() {
        let mut maze = Maze::new(CoordXY::new(0, 0).unwrap(), CoordXY::new(7, 7).unwrap());
        maze.data[0].set_north(true);
        let mut cell = maze.cell(CoordXY::new(0, 0).unwrap());
        assert!(cell.north());
        cell.set_east(true);
        // Since `cell` is just a copy, set_east does not affect the original object
        assert!(!maze.cell(CoordXY::new(0, 0).unwrap()).east());
    }
    #[test]
    fn maze_mutable_cell() {
        let mut maze = Maze::new(CoordXY::new(0, 0).unwrap(), CoordXY::new(7, 7).unwrap());
        maze.data[0].set_north(true);
        let cell = maze.mutable_cell(CoordXY::new(0, 0).unwrap());
        assert!(cell.north());
        cell.set_east(true);
        // Since `cell` is a mutable reference, set_east affects the original object
        assert!(maze.mutable_cell(CoordXY::new(0, 0).unwrap()).east());
    }
    #[test]
    fn maze_set_cell_state() {
        let mut maze = Maze::new(CoordXY::new(0, 0).unwrap(), CoordXY::new(7, 7).unwrap());
        maze.set_cell_state(CoordXY::new(0, 0).unwrap(), Direction::North, true);
        assert!(maze
            .cell_by_x_y(Coord1D::new(0).unwrap(), Coord1D::new(0).unwrap())
            .north());
        assert!(maze
            .cell_by_x_y(Coord1D::new(0).unwrap(), Coord1D::new(1).unwrap())
            .south());
    }
    #[test]
    fn maze_load() {
        let maze = Maze::load_from_str(MAZE_STR);
        assert!(maze
            .cell_by_x_y(Coord1D::new(0).unwrap(), Coord1D::new(0).unwrap())
            .north());
        assert!(maze
            .cell_by_x_y(Coord1D::new(1).unwrap(), Coord1D::new(0).unwrap())
            .north());
        assert!(maze
            .cell_by_x_y(Coord1D::new(1).unwrap(), Coord1D::new(0).unwrap())
            .east());
    }
}
