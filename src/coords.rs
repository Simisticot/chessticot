use std::ops;

use crate::PieceKind;

#[derive(PartialEq, Hash, Eq, Debug, Clone)]
pub enum ChessMove {
    RegularMove(Move),
    PawnSkip(Move),
    CastleLeft,
    CastleRight,
    EnPassant(Move, Coords),
    Promotion(Move, PieceKind),
}

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub struct Move {
    pub origin: Coords,
    pub destination: Coords,
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

impl Coords {
    pub fn is_in_bounds(&self) -> bool {
        self.x < 8 && self.x >= 0 && self.y < 8 && self.y >= 0
    }
}

impl ops::Add<Direction> for Coords {
    type Output = Coords;
    fn add(self, dir: Direction) -> Coords {
        Coords {
            x: self.x + dir.dx,
            y: self.y + dir.dy,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Direction {
    pub dx: isize,
    pub dy: isize,
}

impl ops::Mul<isize> for Direction {
    type Output = Direction;
    fn mul(self, rhs: isize) -> Self::Output {
        Direction {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
        }
    }
}
