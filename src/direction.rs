use ultraviolet::{IVec3, Vec3};
use crate::block::properties::PropertyEnum;

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

    pub fn ordinal_bitwise(&self) -> u32 {
        match self {
            Direction::North =>     0b1,
            Direction::South =>     0b10,
            Direction::East =>      0b100,
            Direction::West =>      0b1000,
            Direction::Up =>        0b10000,
            Direction::Down =>      0b100000,
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
    pub fn name(&self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::South => "South",
            Direction::East => "East",
            Direction::West => "West",
            Direction::Up => "Up",
            Direction::Down => "Down",
        }
    }
}

impl PropertyEnum for Direction {
    fn get_values() -> Vec<u32> {
        DIRECTIONS.iter().map(|dir| dir.ordinal() as u32).collect()
    }

    fn name_value(value: u32) -> String {
        DIRECTIONS[value as usize].name().to_string()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DirectionAll {
    North,
    South,
    East,
    West,
    Up,
    Down,
    NE,
    NW,
    NU,
    ND,
    SE,
    SW,
    SU,
    SD,
    EU,
    ED,
    WU,
    WD,
    NEU,
    NED,
    NWU,
    NWD,
    SEU,
    SED,
    SWU,
    SWD,
}

pub static DIRECTIONS_ALL: [DirectionAll; 26] = [
    DirectionAll::North,
    DirectionAll::South,
    DirectionAll::East,
    DirectionAll::West,
    DirectionAll::Up,
    DirectionAll::Down,
    DirectionAll::NE,
    DirectionAll::NW,
    DirectionAll::NU,
    DirectionAll::ND,
    DirectionAll::SE,
    DirectionAll::SW,
    DirectionAll::SU,
    DirectionAll::SD,
    DirectionAll::EU,
    DirectionAll::ED,
    DirectionAll::WU,
    DirectionAll::WD,
    DirectionAll::NEU,
    DirectionAll::NED,
    DirectionAll::NWU,
    DirectionAll::NWD,
    DirectionAll::SEU,
    DirectionAll::SED,
    DirectionAll::SWU,
    DirectionAll::SWD,
];

impl DirectionAll {
    pub fn get_int_vector(&self) -> IVec3 {
        match self {
            DirectionAll::North => IVec3::new(-1, 0, 0),
            DirectionAll::South => IVec3::new(1, 0, 0),
            DirectionAll::East => IVec3::new(0, 0, -1),
            DirectionAll::West => IVec3::new(0, 0, 1),
            DirectionAll::Up => IVec3::new(0, 1, 0),
            DirectionAll::Down => IVec3::new(0, -1, 0),
            DirectionAll::NE => IVec3::new(-1, 0, -1),
            DirectionAll::NW => IVec3::new(-1, 0, 1),
            DirectionAll::NU => IVec3::new(-1, 1, 0),
            DirectionAll::ND => IVec3::new(-1, -1, 0),
            DirectionAll::SE => IVec3::new(1, 0, -1),
            DirectionAll::SW => IVec3::new(1, 0, 1),
            DirectionAll:: SU => IVec3::new(1, 1, 0),
            DirectionAll:: SD => IVec3::new(1, -1, 0),
            DirectionAll:: EU => IVec3::new(0, 1, -1),
            DirectionAll::ED => IVec3::new(0, -1, -1),
            DirectionAll::WU => IVec3::new(0, 1, 1),
            DirectionAll::WD => IVec3::new(0, -1, 1),
            DirectionAll::NEU => IVec3::new(-1, 1, -1),
            DirectionAll::NED => IVec3::new(-1, -1, -1),
            DirectionAll::NWU => IVec3::new(-1, 1, 1),
            DirectionAll::NWD => IVec3::new(-1, -1, 1),
            DirectionAll::SEU => IVec3::new(1, 1, -1),
            DirectionAll::SED => IVec3::new(1, -1, -1),
            DirectionAll:: SWU => IVec3::new(1, 1, 1),
            DirectionAll:: SWD => IVec3::new(1, -1, 1),
        }
    }

    pub fn get_float_vector(&self) -> Vec3 {
        match self {
            DirectionAll::North => Vec3::new(-1., 0., 0.),
            DirectionAll::South => Vec3::new(1., 0., 0.),
            DirectionAll::East => Vec3::new(0., 0., -1.),
            DirectionAll::West => Vec3::new(0., 0., 1.),
            DirectionAll::Up => Vec3::new(0., 1., 0.),
            DirectionAll::Down => Vec3::new(0., -1., 0.),
            DirectionAll::NE => Vec3::new(-1., 0., -1.),
            DirectionAll::NW => Vec3::new(-1., 0., 1.),
            DirectionAll::NU => Vec3::new(-1., 1., 0.),
            DirectionAll::ND => Vec3::new(-1., -1., 0.),
            DirectionAll::SE => Vec3::new(1., 0., -1.),
            DirectionAll::SW => Vec3::new(1., 0., 1.),
            DirectionAll:: SU => Vec3::new(1., 1., 0.),
            DirectionAll:: SD => Vec3::new(1., -1., 0.),
            DirectionAll:: EU => Vec3::new(0., 1., -1.),
            DirectionAll::ED => Vec3::new(0., -1., -1.),
            DirectionAll::WU => Vec3::new(0., 1., 1.),
            DirectionAll::WD => Vec3::new(0., -1., 1.),
            DirectionAll::NEU => Vec3::new(-1., 1., -1.),
            DirectionAll::NED => Vec3::new(-1., -1., -1.),
            DirectionAll::NWU => Vec3::new(-1., 1., 1.),
            DirectionAll::NWD => Vec3::new(-1., -1., 1.),
            DirectionAll::SEU => Vec3::new(1., 1., -1.),
            DirectionAll::SED => Vec3::new(1., -1., -1.),
            DirectionAll:: SWU => Vec3::new(1., 1., 1.),
            DirectionAll:: SWD => Vec3::new(1., -1., 1.),
        }
    }

    pub fn ordinal(&self) -> usize {
        match self {
            DirectionAll::North => 0,
            DirectionAll::South => 1,
            DirectionAll::East => 2,
            DirectionAll::West => 3,
            DirectionAll::Up => 4,
            DirectionAll::Down => 5,
            DirectionAll::NE => 6,
            DirectionAll::NW => 7,
            DirectionAll::NU => 8,
            DirectionAll::ND => 9,
            DirectionAll::SE => 10,
            DirectionAll::SW => 11,
            DirectionAll:: SU => 12,
            DirectionAll:: SD => 13,
            DirectionAll:: EU => 14,
            DirectionAll::ED => 15,
            DirectionAll::WU => 16,
            DirectionAll::WD => 17,
            DirectionAll::NEU => 18,
            DirectionAll::NED => 19,
            DirectionAll::NWU => 20,
            DirectionAll::NWD => 21,
            DirectionAll::SEU => 22,
            DirectionAll::SED => 23,
            DirectionAll:: SWU => 24,
            DirectionAll:: SWD => 25,
        }
    }

    pub fn ordinal_bitwise(&self) -> u32 {
        match self {
            DirectionAll::North => 1 << 0,
            DirectionAll::South => 1 << 1,
            DirectionAll::East => 1 << 2,
            DirectionAll::West => 1 << 3,
            DirectionAll::Up => 1 << 4,
            DirectionAll::Down => 1 << 5,
            DirectionAll::NE => 1 << 6,
            DirectionAll::NW => 1 << 7,
            DirectionAll::NU => 1 << 8,
            DirectionAll::ND => 1 << 9,
            DirectionAll::SE => 1 << 10,
            DirectionAll::SW => 1 << 11,
            DirectionAll:: SU => 1 << 12,
            DirectionAll:: SD => 1 << 13,
            DirectionAll:: EU => 1 << 14,
            DirectionAll::ED => 1 << 15,
            DirectionAll::WU => 1 << 16,
            DirectionAll::WD => 1 << 17,
            DirectionAll::NEU => 1 << 18,
            DirectionAll::NED => 1 << 19,
            DirectionAll::NWU => 1 << 20,
            DirectionAll::NWD => 1 << 21,
            DirectionAll::SEU => 1 << 22,
            DirectionAll::SED => 1 << 23,
            DirectionAll:: SWU => 1 << 24,
            DirectionAll:: SWD => 1 << 25,
        }
    }
}
