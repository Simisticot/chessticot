use std::ops;

#[derive(PartialEq, Debug)]
pub struct Move {
    pub origin: Coords,
    pub destination: Coords,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

impl Coords {
    pub fn is_in_bounds(&self) -> bool {
        self.x < 8 && self.x >= 0 && self.y < 8 && self.y >= 0
    }
    pub fn raycast(&self, direction: &Direction) -> Vec<Coords> {
        let mut squares = vec![];
        // for instead of loop to avoid potential infinite loop
        for i in 1..8 {
            let next_square = *self + (*direction * i);
            if !next_square.is_in_bounds() {
                break;
            }
            squares.push(next_square);
        }
        squares
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
