mod coords;
mod piece;

pub use crate::coords::{ChessMove, Coords, Direction, Move};
pub use crate::piece::{Piece, PieceColor, PieceKind};
use core::panic;
use std::usize;

pub struct History {
    white_king_moved: bool,
    white_left_rook_moved: bool,
    white_right_rook_moved: bool,
    black_king_moved: bool,
    black_left_rook_moved: bool,
    black_right_rook_moved: bool,
    pub en_passant_on: Option<Coords>,
}
impl History {
    pub fn new() -> History {
        History {
            white_king_moved: false,
            white_left_rook_moved: false,
            white_right_rook_moved: false,
            black_king_moved: false,
            black_left_rook_moved: false,
            black_right_rook_moved: false,
            en_passant_on: None,
        }
    }

    pub fn king_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_king_moved,
            PieceColor::Black => &self.black_king_moved,
        }
    }

    pub fn right_rook_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_right_rook_moved,
            PieceColor::Black => &self.black_right_rook_moved,
        }
    }

    pub fn left_rook_moved(&self, color: &PieceColor) -> &bool {
        match color {
            PieceColor::White => &self.white_left_rook_moved,
            PieceColor::Black => &self.black_left_rook_moved,
        }
    }
}

pub struct Game {
    pub board: Vec<Vec<Option<Piece>>>,
    pub to_move: PieceColor,
    pub checkmated: Option<PieceColor>,
    pub history: History,
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
            checkmated: None,
            history: History::new(),
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
            checkmated: None,
            history: History::new(),
        }
    }
    pub fn make_move(&mut self, chess_move: &ChessMove) {
        if is_move_legal(&self.board, chess_move, &self.to_move, &self.history) {
            execute_move(&mut self.board, chess_move, &self.to_move);
            if let ChessMove::PawnSkip(movement) = chess_move {
                self.history.en_passant_on = Some(Coords {
                    x: movement.origin.x,
                    y: (movement.origin.y + movement.destination.y) / 2 as isize,
                });
            } else {
                self.history.en_passant_on = None;
            }
            self.to_move = self.to_move.opposite();
            if is_checkmate(&self.board, &self.to_move, &self.history) {
                self.checkmated = Some(self.to_move.clone());
            }
        }
    }
}

fn execute_move(board: &mut Vec<Vec<Option<Piece>>>, chess_move: &ChessMove, color: &PieceColor) {
    match chess_move {
        ChessMove::RegularMove(coordinates) => {
            move_piece(board, coordinates.origin, coordinates.destination);
        }
        ChessMove::PawnSkip(coordinates) => {
            move_piece(board, coordinates.origin, coordinates.destination);
        }
        ChessMove::CastleLeft => {
            castle_left(board, color);
        }
        ChessMove::CastleRight => {
            castle_right(board, color);
        }
        ChessMove::EnPassant(movement, pawn_taken) => {
            move_piece(board, movement.origin, movement.destination);
            take_piece_at(board, *pawn_taken);
        }
        ChessMove::Promotion(movement, promoted_to) => {
            take_piece_at(board, movement.origin);
            put_piece_at(
                board,
                Piece {
                    kind: *promoted_to,
                    color: color.clone(),
                },
                movement.destination,
            );
        }
    }
}

fn castle_left(board: &mut Vec<Vec<Option<Piece>>>, color: &PieceColor) {
    let row = match color {
        PieceColor::White => 0,
        PieceColor::Black => 7,
    };
    move_piece(board, Coords { x: 4, y: row }, Coords { x: 2, y: row });
    move_piece(board, Coords { x: 0, y: row }, Coords { x: 3, y: row });
}

fn castle_right(board: &mut Vec<Vec<Option<Piece>>>, color: &PieceColor) {
    let row = match color {
        PieceColor::White => 0,
        PieceColor::Black => 7,
    };
    move_piece(board, Coords { x: 4, y: row }, Coords { x: 6, y: row });
    move_piece(board, Coords { x: 7, y: row }, Coords { x: 5, y: row });
}

fn is_checkmate(board: &Vec<Vec<Option<Piece>>>, to_move: &PieceColor, history: &History) -> bool {
    return is_in_check(to_move, board, history)
        && all_legal_moves(board, to_move, history).len() == 0;
}

fn all_legal_moves(
    board: &Vec<Vec<Option<Piece>>>,
    to_move: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    all_squares()
        .iter()
        .map(|square| legal_moves_from_origin(board, square, to_move, history))
        .flatten()
        .collect()
}

fn all_possible_moves(
    board: &Vec<Vec<Option<Piece>>>,
    to_move: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    all_squares()
        .iter()
        .map(|square| possible_moves_from_origin(board, square, to_move, history))
        .flatten()
        .collect()
}

fn is_move_legal(
    board: &Vec<Vec<Option<Piece>>>,
    chess_move: &ChessMove,
    to_move: &PieceColor,
    history: &History,
) -> bool {
    let origin = match chess_move {
        ChessMove::RegularMove(movement) => movement.origin,
        ChessMove::PawnSkip(movement) => movement.origin,
        ChessMove::EnPassant(movement, _) => movement.origin,
        ChessMove::CastleRight | ChessMove::CastleLeft => {
            let row = match to_move {
                PieceColor::White => 0,
                PieceColor::Black => 7,
            };
            Coords { y: row, x: 4 }
        }
        ChessMove::Promotion(movement, _) => movement.origin,
    };

    legal_moves_from_origin(board, &origin, to_move, history).contains(chess_move)
}

fn opens_own_king(
    board: &Vec<Vec<Option<Piece>>>,
    chess_move: &ChessMove,
    color: &PieceColor,
    history: &History,
) -> bool {
    let mut potential_board = board.clone();
    execute_move(&mut potential_board, chess_move, color);
    is_in_check(color, &potential_board, history)
}

pub fn legal_moves_from_origin(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    to_move: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    possible_moves_from_origin(board, origin, to_move, history)
        .iter()
        .cloned()
        .filter(|chess_move| !opens_own_king(board, chess_move, to_move, history))
        .collect()
}

fn possible_moves_from_origin(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    to_move: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    match piece_at(board, origin) {
        None => Vec::new(),
        Some(piece) => {
            if piece.color == *to_move {
                movement_from_origin(board, origin, piece, history)
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
    history: &History,
) -> Vec<ChessMove> {
    match piece.kind {
        PieceKind::Pawn => pawn_from(board, origin, &piece.color, history),
        PieceKind::Rook => rook_from(board, origin, &piece.color),
        PieceKind::Knight => knight_from(board, origin, &piece.color),
        PieceKind::Bishop => bishop_from(board, origin, &piece.color),
        PieceKind::Queen => queen_movement(board, origin, &piece.color),
        PieceKind::King => king_movement(board, origin, &piece.color, history),
    }
}

fn king_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    origin_color: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    let mut moves = projected_movement(board, origin, eight_degrees(), origin_color, Some(1));
    let row = match origin_color {
        PieceColor::White => 0,
        PieceColor::Black => 7,
    };
    if piece_at(board, &Coords { y: row, x: 5 }).is_none()
        && piece_at(board, &Coords { y: row, x: 6 }).is_none()
        && piece_at(board, &Coords { y: row, x: 4 }).is_some_and(|piece| {
            piece
                == Piece {
                    kind: PieceKind::King,
                    color: origin_color.clone(),
                }
        })
        && !history.king_moved(origin_color)
        && piece_at(board, &Coords { y: row, x: 7 }).is_some_and(|piece| {
            piece
                == Piece {
                    kind: PieceKind::Rook,
                    color: origin_color.clone(),
                }
        })
        && !history.right_rook_moved(origin_color)
    {
        moves.push(ChessMove::CastleRight);
    }
    if piece_at(board, &Coords { y: row, x: 3 }).is_none()
        && piece_at(board, &Coords { y: row, x: 2 }).is_none()
        && piece_at(board, &Coords { y: row, x: 1 }).is_none()
        && piece_at(board, &Coords { y: row, x: 4 }).is_some_and(|piece| {
            piece
                == Piece {
                    kind: PieceKind::King,
                    color: origin_color.clone(),
                }
        })
        && !history.king_moved(origin_color)
        && piece_at(board, &Coords { y: row, x: 0 }).is_some_and(|piece| {
            piece
                == Piece {
                    kind: PieceKind::Rook,
                    color: origin_color.clone(),
                }
        })
        && !history.left_rook_moved(origin_color)
    {
        moves.push(ChessMove::CastleLeft);
    }

    moves
}

fn queen_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
) -> Vec<ChessMove> {
    projected_movement(board, origin, eight_degrees(), color, None)
}

fn bishop_from(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
) -> Vec<ChessMove> {
    projected_movement(board, origin, inter_cards(), color, None)
}

fn knight_from(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
) -> Vec<ChessMove> {
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
    let potential_moves = directions.iter().map(|direction| {
        ChessMove::RegularMove(Move {
            origin: origin.clone(),
            destination: *origin + *direction,
        })
    });
    potential_moves
        .into_iter()
        .filter(|chess_move| match chess_move {
            ChessMove::RegularMove(coordinates) => {
                coordinates.destination.is_in_bounds()
                    && piece_at(board, &coordinates.destination)
                        .is_none_or(|piece| &piece.color != color)
            }
            _ => false,
        })
        .collect()
}

fn rook_from(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
) -> Vec<ChessMove> {
    projected_movement(board, origin, cards(), color, None)
}

fn projected_movement(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    directions: Vec<Direction>,
    origin_color: &PieceColor,
    limit: Option<isize>,
) -> Vec<ChessMove> {
    directions
        .iter()
        .map(|dir| raycast(board, origin, dir, origin_color, limit))
        .flatten()
        .map(|destination| {
            ChessMove::RegularMove(Move {
                origin: origin.clone(),
                destination,
            })
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

fn en_passant_from(origin: &Coords, color: &PieceColor, history: &History) -> Option<ChessMove> {
    match history.en_passant_on {
        None => None,
        Some(coordinates) => {
            for candidate in vec![
                coordinates
                    + Direction {
                        dx: 1,
                        dy: color.opposite().pawn_orientation(),
                    },
                coordinates
                    + Direction {
                        dx: -1,
                        dy: color.opposite().pawn_orientation(),
                    },
            ] {
                if candidate.is_in_bounds() && candidate == *origin {
                    return Some(ChessMove::EnPassant(
                        Move {
                            origin: origin.clone(),
                            destination: coordinates.clone(),
                        },
                        coordinates
                            + Direction {
                                dx: 0,
                                dy: color.opposite().pawn_orientation(),
                            },
                    ));
                }
            }
            None
        }
    }
}

fn pawn_from(
    board: &Vec<Vec<Option<Piece>>>,
    origin: &Coords,
    color: &PieceColor,
    history: &History,
) -> Vec<ChessMove> {
    let mut legal_moves = vec![];
    let forward = Direction {
        dx: 0,
        dy: color.pawn_orientation(),
    };
    let ahead_one = *origin + forward;
    let ahead_two = ahead_one + forward;

    if !ahead_one.is_in_bounds() {
        return legal_moves;
    }

    if piece_at(board, &ahead_one).is_none() {
        legal_moves.push(ChessMove::RegularMove(Move {
            origin: origin.clone(),
            destination: ahead_one,
        }));
        if !ahead_two.is_in_bounds() {
            return legal_moves;
        };
        if (origin.y == 1 || origin.y == 6) && piece_at(board, &ahead_two).is_none() {
            legal_moves.push(ChessMove::PawnSkip(Move {
                origin: origin.clone(),
                destination: ahead_two,
            }));
        }
    }

    vec![
        Coords {
            x: origin.x + 1,
            y: origin.y + color.pawn_orientation(),
        },
        Coords {
            x: origin.x - 1,
            y: origin.y + color.pawn_orientation(),
        },
    ]
    .iter()
    .for_each(|diagonal| {
        if diagonal.is_in_bounds() {
            match piece_at(board, &diagonal) {
                None => {}
                Some(piece) => {
                    if piece.color == color.opposite() {
                        legal_moves.push(ChessMove::RegularMove(Move {
                            origin: origin.clone(),
                            destination: *diagonal,
                        }));
                    }
                }
            }
        }
    });
    if let Some(en_passant) = en_passant_from(origin, color, &history) {
        legal_moves.push(en_passant);
    }
    legal_moves
        .iter()
        .map(|pawn_move| match pawn_move {
            ChessMove::RegularMove(movement) => {
                if movement.destination.y == color.opposite().homerow() {
                    PieceKind::promoteable()
                        .map(|promotable_kind| {
                            ChessMove::Promotion(movement.clone(), promotable_kind.clone())
                        })
                        .collect()
                } else {
                    vec![pawn_move.clone()]
                }
            }
            ChessMove::PawnSkip(_) => vec![pawn_move.clone()],
            ChessMove::EnPassant(_, _) => vec![pawn_move.clone()],
            _ => panic!("Pawn moves should only be regular, skip or en passant"),
        })
        .flatten()
        .collect()
}

fn is_in_check(color: &PieceColor, board: &Vec<Vec<Option<Piece>>>, history: &History) -> bool {
    match king_position(color, &board) {
        None => false,
        Some(loc) => is_attacked(color, &board, &loc, history),
    }
}

fn is_attacked(
    color: &PieceColor,
    board: &Vec<Vec<Option<Piece>>>,
    square: &Coords,
    history: &History,
) -> bool {
    all_possible_moves(board, &color.opposite(), history)
        .iter()
        .map(|chess_move| match chess_move {
            ChessMove::RegularMove(coordinates) => &coordinates.destination == square,
            ChessMove::EnPassant(_, taken) => taken == square,
            _ => false,
        })
        .any(|attacks_square| attacks_square)
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: Coords { y: 2, x: 4 }
                }),
                ChessMove::PawnSkip(Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                })
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: Coords { y: 3, x: 4 }
            })]
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: Coords { y: 3, x: 4 }
                }),
                ChessMove::RegularMove(Move {
                    origin: pawn_location,
                    destination: opposing_pawn_location
                })
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: opposing_pawn_location
            })]
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: Coords { y: 2, x: 4 }
            })]
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
            legal_moves_from_origin(&game.board, &pawn_location, &game.to_move, &game.history),
            vec![ChessMove::RegularMove(Move {
                origin: pawn_location,
                destination: capture_location
            })]
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
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: rook_location,
                    destination: Coords { y: 4, x: j },
                }));
            }
        }
        for i in 0..8 {
            if i != 4 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: rook_location,
                    destination: Coords { x: 4, y: i },
                }));
            }
        }

        let legal_move_set: HashSet<ChessMove, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &rook_location, &game.to_move, &game.history),
        );
        let diff: HashSet<&ChessMove, RandomState> =
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
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: up,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: down,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: left,
            }),
            ChessMove::RegularMove(Move {
                origin: rook_location,
                destination: right,
            }),
        ];

        assert_eq!(
            legal_moves_from_origin(&game.board, &rook_location, &game.to_move, &game.history),
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
            legal_moves_from_origin(&game.board, &rook_location, &game.to_move, &game.history),
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

        let legal_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            vec![
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 4 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 5 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 5, x: 2 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 4, x: 1 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 4 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 5 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                }),
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move, &game.history)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
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

        let legal_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            vec![
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 2, x: 1 },
                }),
                ChessMove::RegularMove(Move {
                    origin: knight_location,
                    destination: Coords { y: 1, x: 2 },
                }),
            ]
            .iter()
            .cloned(),
        );

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move, &game.history)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
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
            legal_moves_from_origin(&game.board, &knight_location, &game.to_move, &game.history)
                .len(),
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
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: bishop_location,
                    destination: Coords { y: j, x: j },
                }));
            }
        }

        for i in 0..7 {
            if i != 3 {
                legal_moves.push(ChessMove::RegularMove(Move {
                    origin: bishop_location,
                    destination: Coords { y: 6 - i, x: i },
                }));
            }
        }

        let legal_move_set: HashSet<ChessMove, RandomState> =
            HashSet::from_iter(legal_moves.iter().cloned());
        let found_move_set: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &bishop_location, &game.to_move, &game.history)
                .iter()
                .cloned(),
        );
        let difference_set: HashSet<&ChessMove, RandomState> = legal_move_set
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
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 4 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 3, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 3 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 3 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 4 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 4, x: 2 },
            }),
            ChessMove::RegularMove(Move {
                origin: king_location.clone(),
                destination: Coords { y: 2, x: 4 },
            }),
        ]);

        let found_moves: HashSet<ChessMove, RandomState> = HashSet::from_iter(
            legal_moves_from_origin(&game.board, &king_location, &game.to_move, &game.history)
                .iter()
                .cloned(),
        );

        let diff: HashSet<&ChessMove, RandomState> =
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
            legal_moves_from_origin(&game.board, &king_location, &game.to_move, &game.history)
                .len(),
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
            &ChessMove::RegularMove(Move {
                origin: king_location,
                destination: Coords { y: 0, x: 1 },
            }),
            &game.to_move,
            &game.history
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
            &ChessMove::RegularMove(Move {
                origin: king_location,
                destination: Coords { y: 0, x: 1 },
            }),
            &game.to_move,
            &game.history
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
        assert!(is_attacked(
            &PieceColor::White,
            &game.board,
            &king_location,
            &game.history
        ));
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
        assert!(is_in_check(&PieceColor::White, &game.board, &game.history));
    }

    #[test]
    fn detects_checkmate() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[1][1] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::Black,
        });
        game.board[2][2] = Some(Piece {
            kind: PieceKind::Queen,
            color: PieceColor::Black,
        });
        assert!(is_checkmate(&game.board, &game.to_move, &game.history));
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
    fn can_castle_right() {
        let mut game = Game::empty();

        game.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        assert!(
            piece_at(&game.board, &Coords { y: 0, x: 5 }).is_none()
                && piece_at(&game.board, &Coords { y: 0, x: 6 }).is_none()
        );
        assert!(legal_moves_from_origin(
            &game.board,
            &Coords { y: 0, x: 4 },
            &game.to_move,
            &game.history
        )
        .contains(&ChessMove::CastleRight));
        assert!(is_move_legal(
            &game.board,
            &ChessMove::CastleRight,
            &game.to_move,
            &game.history
        ))
    }

    #[test]
    fn castle_right() {
        let mut game = Game::empty();

        game.board[0][4] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        game.board[0][7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PieceColor::White,
        });
        execute_move(&mut game.board, &ChessMove::CastleRight, &PieceColor::White);
        assert!(piece_at(&game.board, &Coords { y: 0, x: 4 }).is_none());
        assert!(piece_at(&game.board, &Coords { y: 0, x: 7 }).is_none());
        assert!(
            piece_at(&game.board, &Coords { y: 0, x: 6 }).is_some_and(|piece| piece
                == Piece {
                    kind: PieceKind::King,
                    color: PieceColor::White
                })
        );
        assert!(
            piece_at(&game.board, &Coords { y: 0, x: 5 }).is_some_and(|piece| piece
                == Piece {
                    kind: PieceKind::Rook,
                    color: PieceColor::White
                })
        );
    }

    #[test]
    fn make_move() {
        let mut game = Game::empty();

        game.board[0][0] = Some(Piece {
            kind: PieceKind::King,
            color: PieceColor::White,
        });
        let king_location = Coords { x: 0, y: 0 };
        game.make_move(&ChessMove::RegularMove(Move {
            origin: king_location,
            destination: Coords { x: 0, y: 1 },
        }));
        assert!(piece_at(&game.board, &king_location).is_none());
        assert_eq!(
            piece_at(&game.board, &Coords { x: 0, y: 1 }).unwrap().kind,
            PieceKind::King
        );
    }

    #[test]
    fn scholars_mate() {
        let mut game = Game::start();

        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { x: 4, y: 1 },
            destination: Coords { x: 4, y: 3 },
        }));
        assert!(game.to_move == PieceColor::Black);
        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { x: 4, y: 6 },
            destination: Coords { x: 4, y: 4 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 3, y: 0 },
            destination: Coords { x: 7, y: 4 },
        }));
        assert!(game.to_move == PieceColor::Black);
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 1, y: 7 },
            destination: Coords { x: 2, y: 5 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 5, y: 0 },
            destination: Coords { x: 2, y: 3 },
        }));
        assert!(game.to_move == PieceColor::Black);
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 6, y: 7 },
            destination: Coords { x: 5, y: 5 },
        }));
        game.make_move(&ChessMove::RegularMove(Move {
            origin: Coords { x: 7, y: 4 },
            destination: Coords { x: 5, y: 6 },
        }));
        assert!(game.to_move == PieceColor::Black);

        assert!(game.checkmated == Some(PieceColor::Black));
    }

    #[test]
    fn pawn_skip_is_legal() {
        let game = Game::start();
        assert!(is_move_legal(
            &game.board,
            &ChessMove::PawnSkip(Move {
                origin: Coords { x: 4, y: 1 },
                destination: Coords { x: 4, y: 3 }
            }),
            &game.to_move,
            &game.history
        ))
    }

    #[test]
    fn en_passant_right() {
        let mut game = Game::empty();
        game.board[1][1] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][0] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 1 },
            destination: Coords { y: 3, x: 1 },
        }));
        let black_pawn_location = Coords { y: 3, x: 0 };
        let ep = ChessMove::EnPassant(
            Move {
                origin: black_pawn_location,
                destination: Coords { y: 2, x: 1 },
            },
            Coords { y: 3, x: 1 },
        );
        assert!(game.history.en_passant_on == Some(Coords { y: 2, x: 1 }));
        assert!(legal_moves_from_origin(
            &game.board,
            &black_pawn_location,
            &game.to_move,
            &game.history
        )
        .contains(&ep));
        assert!(is_move_legal(
            &game.board,
            &ep,
            &game.to_move,
            &game.history
        ))
    }
    #[test]
    fn en_passant_left() {
        let mut game = Game::empty();
        game.board[1][1] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[3][2] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 1 },
            destination: Coords { y: 3, x: 1 },
        }));
        let black_pawn_location = Coords { y: 3, x: 2 };
        let ep = ChessMove::EnPassant(
            Move {
                origin: black_pawn_location,
                destination: Coords { y: 2, x: 1 },
            },
            Coords { y: 3, x: 1 },
        );
        assert!(game.history.en_passant_on == Some(Coords { y: 2, x: 1 }));
        assert!(legal_moves_from_origin(
            &game.board,
            &black_pawn_location,
            &game.to_move,
            &game.history
        )
        .contains(&ep));
        assert!(is_move_legal(
            &game.board,
            &ep,
            &game.to_move,
            &game.history
        ))
    }
    #[test]
    fn no_en_passant_from_accross_the_board() {
        let mut game = Game::empty();
        game.board[1][4] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::White,
        });
        game.board[7][2] = Some(Piece {
            kind: PieceKind::Pawn,
            color: PieceColor::Black,
        });
        game.make_move(&ChessMove::PawnSkip(Move {
            origin: Coords { y: 1, x: 4 },
            destination: Coords { y: 3, x: 4 },
        }));

        assert!(game.history.en_passant_on == Some(Coords { y: 2, x: 4 }));
        assert!(!is_move_legal(
            &game.board,
            &ChessMove::EnPassant(
                Move {
                    origin: Coords { y: 7, x: 2 },
                    destination: Coords { y: 2, x: 4 }
                },
                Coords { y: 3, x: 4 }
            ),
            &game.to_move,
            &game.history
        ))
    }
}
