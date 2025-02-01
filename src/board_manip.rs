use crate::Coords;
use crate::Piece;

pub fn move_piece(board: &mut Vec<Vec<Option<Piece>>>, origin: Coords, dest: Coords) {
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
