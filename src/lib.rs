mod coords;
mod piece;

pub use crate::coords::{Coords, Direction, Move};
pub use crate::piece::{Piece, PieceColor, PieceKind};
use std::usize;

pub struct Game {
    pub board: Vec<Vec<Option<Piece>>>,
    pub to_move: PieceColor,
}

impl Game {
    pub fn start() -> Game {
        let mut board = Vec::new();
        for i in 0..8 {
            let mut row = Vec::new();
            for j in 0..8 {
                row.push(Piece::from_initial_position(j, i));
            }
            board.push(row);
        }
        Game {
            board,
            to_move: PieceColor::White,
        }
    }

    pub fn empty() -> Game {
        let mut board = Vec::new();
        for _ in 0..8 {
            let mut row = Vec::new();
            for _ in 0..8 {
                row.push(None);
            }
            board.push(row);
        }
        Game {
            board,
            to_move: PieceColor::White,
        }
    }
    pub fn make_move(&mut self, chess_move: &Move) {
        if is_move_legal(&self.board, chess_move, &self.to_move) {
            move_piece(&mut self.board, chess_move.origin, chess_move.destination);
            self.to_move = self.to_move.opposite();
        }
    }
}
fn is_move_legal(board: &Vec<Vec<Option<Piece>>>, chess_move: &Move, to_move: &PieceColor) -> bool {
    legal_moves_from_origin(board, &chess_move.origin, to_move).contains(chess_move)
        && !opens_own_king(board, chess_move, to_move)
}

fn opens_own_king(board: &Vec<Vec<Option<Piece>>>, chess_move: &Move, color: &PieceColor) -> bool {
    let mut potential_board = board.clone();
    move_piece(
        &mut potential_board,
        chess_move.origin,
        chess_move.destination,
    );
    is_in_check(color, &potential_board)
}

pub fn legal_moves_from_origin(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    to_move: &PieceColor,
) -> Vec<Move> {
    match piece_at(board, origin) {
        None => Vec::new(),
        Some(piece) => {
            if piece.color == *to_move {
                movement_from_origin(board, origin, piece)
            } else {
                Vec::new()
            }
        }
    }
}

fn movement_from_origin(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    piece: Piece,
) -> Vec<Move> {
    match piece.kind {
        PieceKind::Pawn => pawn_from(board, origin, &piece.color),
        PieceKind::Rook => rook_from(board, origin, &piece.color),
        PieceKind::Knight => knight_from(board, origin, &piece.color),
        PieceKind::Bishop => bishop_from(board, origin, &piece.color),
        PieceKind::Queen => queen_movement(board, origin, &piece.color),
        PieceKind::King => king_movement(board, origin, &piece.color),
    }
}

fn king_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    origin_color: &PieceColor,
) -> Vec<Move> {
    projected_movement(board, origin, eight_degrees(), origin_color, Some(1))
}

fn queen_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
) -> Vec<Move> {
    projected_movement(board, origin, eight_degrees(), color, None)
}

fn bishop_from(board: &Vec<Vec<Option<Piece>>>, origin: &Coords, color: &PieceColor) -> Vec<Move> {
    projected_movement(board, origin, inter_cards(), color, None)
}

fn knight_from(board: &Vec<Vec<Option<Piece>>>, origin: &Coords, color: &PieceColor) -> Vec<Move> {
    let directions: Vec<Direction> = vec![
        Direction { dy: 2, dx: 1 },
        Direction { dy: 2, dx: -1 },
        Direction { dy: 1, dx: 2 },
        Direction { dy: 1, dx: -2 },
        Direction { dy: -2, dx: 1 },
        Direction { dy: -2, dx: -1 },
        Direction { dy: -1, dx: -2 },
        Direction { dy: -1, dx: 2 },
    ];
    let potential_moves = directions.iter().map(|direction| Move {
        origin: origin.clone(),
        destination: *origin + *direction,
    });
    potential_moves
        .into_iter()
        .filter(|chess_move| {
            chess_move.destination.is_in_bounds()
                && piece_at(board, &chess_move.destination)
                    .is_none_or(|piece| &piece.color != color)
        })
        .collect()
}

fn rook_from(board: &Vec<Vec<Option<Piece>>>, origin: &Coords, color: &PieceColor) -> Vec<Move> {
    projected_movement(board, origin, cards(), color, None)
}

fn projected_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    directions: Vec<Direction>,
    origin_color: &PieceColor,
    limit: Option<isize>,
) -> Vec<Move> {
    directions
        .iter()
        .map(|dir| raycast(board, origin, dir, origin_color, limit))
        .flatten()
        .map(|destination| Move {
            origin: origin.clone(),
            destination,
        })
        .collect()
}
pub fn raycast(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    direction: &Direction,
    origin_color: &PieceColor,
    limit: Option<isize>,
) -> Vec<Coords> {
    let limit = limit.unwrap_or(7) + 1;
    let mut squares = vec![];
    // for instead of loop to avoid potential infinite loop
    for i in 1..limit {
        let next_square = *origin + (*direction * i);
        if !next_square.is_in_bounds() {
            break;
        }
        if let Some(piece) = piece_at(board, &next_square) {
            if piece.color == origin_color.opposite() {
                squares.push(next_square);
            }
            break;
        }
        squares.push(next_square);
    }
    squares
}

fn pawn_from(board: &Vec<Vec<Option<Piece>>>, origin: &Coords, color: &PieceColor) -> Vec<Move> {
    let vertical_orientation = match color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
    };
    let mut legal_moves = vec![];
    let forward = Direction {
        dx: 0,
        dy: vertical_orientation,
    };
    let ahead_one = *origin + forward;
    let ahead_two = ahead_one + forward;

    if !ahead_one.is_in_bounds() {
        return legal_moves;
    }

    if piece_at(board, &ahead_one).is_none() {
        legal_moves.push(Move {
            origin: origin.clone(),
            destination: ahead_one,
        });
        if (origin.y == 1 || origin.y == 6) && piece_at(board, &ahead_two).is_none() {
            legal_moves.push(Move {
                origin: origin.clone(),
                destination: ahead_two,
            });
        }
    }

    vec![
        Coords {
            x: origin.x + 1,
            y: origin.y + vertical_orientation,
        },
        Coords {
            x: origin.x - 1,
            y: origin.y + vertical_orientation,
        },
    ]
    .iter()
    .for_each(|diagonal| {
        if diagonal.is_in_bounds() {
            match piece_at(board, &diagonal) {
                None => {}
                Some(piece) => {
                    if piece.color == color.opposite() {
                        legal_moves.push(Move {
                            origin: origin.clone(),
                            destination: *diagonal,
                        });
                    }
                }
            }
        }
    });
    legal_moves
}

fn is_in_check(color: &PieceColor, board: &Vec<Vec<Option<Piece>>>) -> bool {
    match king_position(color, &board) {
        None => false,
        Some(loc) => is_attacked(color, &board, &loc),
    }
}

fn is_attacked(color: &PieceColor, board: &Vec<Vec<Option<Piece>>>, square: &Coords) -> bool {
    let mut attacked = false;
    all_squares().iter().for_each(|loc| {
        if legal_moves_from_origin(board, &loc, &color.opposite())
            .iter()
            .map(|chess_move| chess_move.destination)
            .collect::<Vec<_>>()
            .contains(square)
        {
            attacked = true;
        }
    });
    attacked
}

fn all_squares() -> Vec<Coords> {
    let mut squares = Vec::new();
    for i in 0..8 {
        for j in 0..8 {
            squares.push(Coords { y: i, x: j });
        }
    }
    squares
}

fn king_position(color: &PieceColor, board: &Vec<Vec<Option<Piece>>>) -> Option<Coords> {
    for i in 0..8 {
        for j in 0..8 {
            let loc = Coords { y: i, x: j };
            if piece_at(&board, &loc)
                .is_some_and(|piece| piece.kind == PieceKind::King && piece.color == *color)
            {
                return Some(loc);
            }
        }
    }
    None
}

fn move_piece(board: &mut Vec<Vec<Option<Piece>>>, origin: Coords, dest: Coords) {
    if let Some(origin_piece) = take_piece_at(board, origin) {
        put_piece_at(board, origin_piece, dest);
    }
}
pub fn piece_at(board: &Vec<Vec<Option<Piece>>>, loc: &Coords) -> Option<Piece> {
    board[loc.y as usize][loc.x as usize].clone()
}
pub fn take_piece_at(board: &mut Vec<Vec<Option<Piece>>>, loc: Coords) -> Option<Piece> {
    board[loc.y as usize][loc.x as usize].take()
}
pub fn put_piece_at(board: &mut Vec<Vec<Option<Piece>>>, piece: Piece, loc: Coords) {
    board[loc.y as usize][loc.x as usize] = Some(piece);
}

fn eight_degrees() -> Vec<Direction> {
    let mut directions: Vec<Direction> = vec![];
    directions.append(&mut cards());
    directions.append(&mut inter_cards());
    directions
}

fn inter_cards() -> Vec<Direction> {
    let up_right = Direction { dy: 1, dx: 1 };
    let down_left = Direction { dy: -1, dx: -1 };
    let up_left = Direction { dy: 1, dx: -1 };
    let down_right = Direction { dy: -1, dx: 1 };
    vec![up_right, down_left, up_left, down_right]
}

fn cards() -> Vec<Direction> {
    let up = Direction { dx: 0, dy: 1 };
    let down = Direction { dx: 0, dy: -1 };
    let left = Direction { dx: -1, dy: 0 };
    let right = Direction { dx: 1, dy: 0 };
    vec![up, down, left, right]
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash::RandomState};

    use super::*;

    #[test]
    fn pawn_homerow() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 2, x: 4 }
                },
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                }
            ]
        )
    }

    #[test]
    fn pawn_not_homerow() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![Move {
                origin: pawn_location,
                destination: Coords { y: 3, x: 4 }
            }]
        )
    }

    #[test]
    fn pawn_not_homerow_with_capture() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![
                Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                },
                Move {
                    origin: pawn_location,
                    destination: opposing_pawn_location
                }
            ]
        )
    }

    #[test]
    fn pawn_not_homerow_blocked() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![]
        )
    }

    #[test]
    fn pawn_edge_of_board_horizontal_blocked() {
        let mut game = Game::empty();
        game.board[2][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][7] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 2, x: 7 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![]
        )
    }

    #[test]
    fn pawn_edge_of_board_vertical() {
        let mut game = Game::empty();
        game.board[7][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        let pawn_location = Coords { y: 7, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![]
        )
    }

    #[test]
    fn pawn_not_homerow_with_capture_blocked() {
        let mut game = Game::empty();
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.board[3][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 2, x: 4 };
        let opposing_pawn_location = Coords { y: 3, x: 5 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![Move {
                origin: pawn_location,
                destination: opposing_pawn_location
            }]
        )
    }

    #[test]
    fn pawn_homerow_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![]
        )
    }
    #[test]
    fn pawn_homerow_second_square_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![Move {
                origin: pawn_location,
                destination: Coords { y: 2, x: 4 }
            }]
        )
    }

    #[test]
    fn pawn_homerow_with_capture_blocked() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[2][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.board[2][5] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        let pawn_location = Coords { y: 1, x: 4 };
        let capture_location = Coords { y: 2, x: 5 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move),
            vec![Move {
                origin: pawn_location,
                destination: capture_location
            }]
        )
    }

    #[test]
    fn rook_middle_board() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let mut legal_moves = vec![];

        for j in 0..8 {
            if j != 4 {
                legal_moves.push(Move {
                    origin: rook_location,
                    destination: Coords { y: 4, x: j },
                });
            }
        }
        for i in 0..8 {
            if i != 4 {
                legal_moves.push(Move {
                    origin: rook_location,
                    destination: Coords { x: 4, y: i },
                });
            }
        }

        let legal_move_set: HashSet<Move, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_moves: HashSet<Move, RandomState> = HashSet::from_iter(legal_moves_from_origin(
            &game.board,
            &rook_location,
            &game.to_move,
        ));
        let diff: HashSet<&Move, RandomState> =
            legal_move_set.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn rook_middle_board_boxed_in_opposite_color() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        game.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::Black,
        });
        let rook_location = Coords { y: 4, x: 4 };
        let up = Coords { y: 5, x: 4 };
        let down = Coords { y: 3, x: 4 };
        let left = Coords { y: 4, x: 3 };
        let right = Coords { y: 4, x: 5 };

        let legal_moves = vec![
            Move {
                origin: rook_location,
                destination: up,
            },
            Move {
                origin: rook_location,
                destination: down,
            },
            Move {
                origin: rook_location,
                destination: left,
            },
            Move {
                origin: rook_location,
                destination: right,
            },
        ];

        assert_eq!(
            legal_moves_from_origin(&game.board, &rook_location, &game.to_move),
            legal_moves
        );
    }

    #[test]
    fn rook_middle_board_boxed_in_own_color() {
        let mut game = Game::empty();
        game.board[4][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[5][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[3][4] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[4][5] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        game.board[4][3] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        let rook_location = Coords { y: 4, x: 4 };

        let legal_moves = vec![];

        assert_eq!(
            legal_moves_from_origin(&game.board, &rook_location, &game.to_move),
            legal_moves
        );
    }

    #[test]
    fn knight_middle_board() {
        let mut game = Game::empty();
        game.board[3][3] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 3, x: 3 };

        let legal_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            vec![
                Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 4 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 5 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 2 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 1 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 4 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 5 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                },
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&Move, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn knight_corner() {
        let mut game = Game::empty();
        game.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        let legal_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            vec![
                Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                },
                Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                },
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&Move, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn knight_corner_blocked() {
        let mut game = Game::empty();
        game.board[0][0] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        game.board[1][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        game.board[2][1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::White,
        });
        let knight_location = Coords { y: 0, x: 0 };

        assert_eq!(
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move).len(),
            0
        )
    }

    #[test]
    fn bishob_middle_board() {
        let mut game = Game::empty();
        game.board[3][3] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PieceColor::White,
        });
        let bishop_location = Coords { y: 3, x: 3 };
        let mut legal_moves = vec![];

        for j in 0..8 {
            if j != 3 {
                legal_moves.push(Move {
                    origin: bishop_location,
                    destination: Coords { y: j, x: j },
                });
            }
        }

        for i in 0..7 {
            if i != 3 {
                legal_moves.push(Move {
                    origin: bishop_location,
                    destination: Coords { y: 6 - i, x: i },
                });
            }
        }

        let legal_move_set: HashSet<Move, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_move_set: HashSet<Move, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &bishop_location, &game.to_move)
                .iter()
                .cloned(),
        );
        let difference_set: HashSet<&Move, RandomState> = legal_move_set
            .symmetric_difference(&found_move_set)
            .collect();
        assert_eq!(difference_set, HashSet::new());
    }

    #[test]
    fn king_middle_board() {
        let mut game = Game::empty();
        game.board[3][3] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        let king_location = Coords { y: 3, x: 3 };
        let legal_moves = HashSet::from([
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 4 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 2 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 3 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 3 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 4 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 2 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 2 },
            },
            Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 4 },
            },
        ]);

        let found_moves: HashSet<Move, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &king_location, &game.to_move)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&Move, RandomState> =
            legal_moves.symmetric_difference(&found_moves).collect();

        assert_eq!(diff, HashSet::new())
    }

    #[test]
    fn cannot_move_out_of_turn() {
        let mut game = Game::empty();
        game.board[3][3] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 3, x: 3 };
        assert_eq!(
            legal_moves_from_origin(&game.board, &king_location, &game.to_move).len(),
            0
        );
    }

    #[test]
    fn detects_move_into_check() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        assert!(opens_own_king(
            &game.board,
            &Move {
                origin: king_location,
                destination: Coords { y: 0, x: 1 },
            },
            &game.to_move
        ));
    }

    #[test]
    fn cannot_move_into_check() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 0 };
        assert!(!is_move_legal(
            &game.board,
            &Move {
                origin: king_location,
                destination: Coords { y: 0, x: 1 },
            },
            &game.to_move
        ));
    }

    #[test]
    fn detects_check() {
        let mut game = Game::empty();

        game.board[0][1] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        let king_location = Coords { y: 0, x: 1 };
        assert!(is_attacked(&PieceColor::White, &game.board, &king_location));
    }

    #[test]
    fn detects_attacked_square() {
        let mut game = Game::empty();

        game.board[0][1] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[2][2] = Some(Piece {
            kind: PieceKind::Knight,
            color: PieceColor::Black,
        });
        assert!(is_in_check(&PieceColor::White, &game.board));
    }

    #[test]
    fn finds_king() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        assert_eq!(
            king_position(&PieceColor::White, &game.board).unwrap(),
            Coords { x: 0, y: 0 }
        )
    }

    #[test]
    fn make_move() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        let king_location = Coords { x: 0, y: 0 };
        game.make_move(&Move {
            origin: king_location,
            destination: Coords { x: 0, y: 1 },
        });
        assert!(piece_at(&game.board, &king_location).is_none());
        assert_eq!(
            piece_at(&game.board, &Coords { x: 0, y: 1 }).unwrap().kind,
            PieceKind::King
        );
    }
}
