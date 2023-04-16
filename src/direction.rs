use ultraviolet::{IVec3, Vec3};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

pub static DIRECTIONS: [Direction; 6] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
    Direction::Up,
    Direction::Down,
];

impl Direction {
    pub fn get_int_vector(&self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(-1, 0, 0),
            Direction::South => IVec3::new(1, 0, 0),
            Direction::East => IVec3::new(0, 0, -1),
            Direction::West => IVec3::new(0, 0, 1),
            Direction::Up => IVec3::new(0, 1, 0),
            Direction::Down => IVec3::new(0, -1, 0),
        }
    }

    pub fn get_float_vector(&self) -> Vec3 {
        match self {
            Direction::North => Vec3::new(-1.0, 0.0, 0.0),
            Direction::South => Vec3::new(1.0, 0.0, 0.0),
            Direction::East => Vec3::new(0.0, 0.0, -1.0),
            Direction::West => Vec3::new(0.0, 0.0, 1.0),
            Direction::Up => Vec3::new(0.0, 1.0, 0.0),
            Direction::Down => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    pub fn ordinal(&self) -> usize {
        match self {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3,
            Direction::Up => 4,
            Direction::Down => 5,
        }
    }

    pub fn reverse(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn reverse_horizontal(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            _ => *self
        }
    }

    pub fn cw(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            _ => *self
        }
    }
    pub fn ccw(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            _ => *self
        }
    }
}
