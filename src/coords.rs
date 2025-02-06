use std::ops;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

impl Coords {
    pub fn is_in_bounds(&self) -> bool {
        self.x < 8 && self.x >= 0 && self.y < 8 && self.y >= 0
    }

    pub fn from_algebraic(square: &str) -> Coords {
        assert!(square.len() == 2);

        let x = match square
            .chars()
            .nth(0)
            .expect("algebraic coordinates should be 2 characters long (asserted above)")
        {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("algebraic file should be a through h"),
        };

        let y = square
            .chars()
            .nth(1)
            .expect("algebraic coordinates should be 2 characters long (asserted above)")
            .to_digit(10)
            .expect("algebraic rank should be a digit from 1 to 8")
            - 1;

        assert!(y <= 7);

        Coords {
            x: x as isize,
            y: y as isize,
        }
    }
}
pub fn all_squares() -> Vec<Coords> {
    let mut squares = Vec::new();
    for i in 0..8 {
        for j in 0..8 {
            squares.push(Coords { y: i, x: j });
        }
    }
    squares
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

pub fn eight_degrees() -> Vec<Direction> {
    let mut directions: Vec<Direction> = vec![];
    directions.append(&mut cards());
    directions.append(&mut inter_cards());
    directions
}

pub fn inter_cards() -> Vec<Direction> {
    let up_right = Direction { dy: 1, dx: 1 };
    let down_left = Direction { dy: -1, dx: -1 };
    let up_left = Direction { dy: 1, dx: -1 };
    let down_right = Direction { dy: -1, dx: 1 };
    vec![up_right, down_left, up_left, down_right]
}

pub fn cards() -> Vec<Direction> {
    let up = Direction { dx: 0, dy: 1 };
    let down = Direction { dx: 0, dy: -1 };
    let left = Direction { dx: -1, dy: 0 };
    let right = Direction { dx: 1, dy: 0 };
    vec![up, down, left, right]
}

#[cfg(test)]
mod tests {
    use crate::Coords;

    #[test]
    fn coord_from_algebraic() {
        assert_eq!(Coords { x: 4, y: 3 }, Coords::from_algebraic("e4"));
    }
}
