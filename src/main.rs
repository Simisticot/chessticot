use chessticot::{
    legal_moves_from_origin, piece_at, ChessMove, Coords, Game, Move, PieceColor, PieceKind,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Widget,
    },
    DefaultTerminal, Frame,
};
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::init().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    game: Game,
    cursor: Coords,
    selected_square: Option<Coords>,
    highlighted_destinations: Vec<Coords>,
    highlighted_castle_left: bool,
    highlighted_castle_right: bool,
}

impl App {
    pub fn init() -> App {
        App {
            exit: false,
            game: Game::start(),
            cursor: Coords { x: 0, y: 0 },
            selected_square: None,
            highlighted_destinations: Vec::new(),
            highlighted_castle_left: false,
            highlighted_castle_right: false,
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') | KeyCode::Left => self.move_cursor(Coords { x: -1, y: 0 }),
            KeyCode::Char('l') | KeyCode::Right => self.move_cursor(Coords { x: 1, y: 0 }),
            KeyCode::Char('k') | KeyCode::Up => self.move_cursor(Coords { x: 0, y: 1 }),
            KeyCode::Char('j') | KeyCode::Down => self.move_cursor(Coords { x: 0, y: -1 }),
            KeyCode::Char(' ') => match self.selected_square {
                None => self.select_square(),
                Some(square) => self.confirm_move(square),
            },
            KeyCode::Esc => self.clear_selection(),
            _ => {}
        }
    }

    fn select_square(&mut self) {
        self.selected_square = Some(self.cursor.clone());
        let legal_moves = legal_moves_from_origin(
            &self.game.board,
            &self.cursor,
            &self.game.to_move,
            &self.game.history,
        );
        self.highlighted_destinations = legal_moves
            .iter()
            .filter_map(|chess_move| {
                let starting_row = self.game.to_move.homerow();
                match chess_move {
                    ChessMove::RegularMove(coordinates) => Some(coordinates.destination),
                    ChessMove::PawnSkip(coordinates) => Some(coordinates.destination),
                    ChessMove::CastleLeft => Some(Coords {
                        y: starting_row,
                        x: 2,
                    }),
                    ChessMove::CastleRight => Some(Coords {
                        y: starting_row,
                        x: 6,
                    }),
                    ChessMove::EnPassant(movement, _) => Some(movement.destination),
                }
            })
            .collect();
        if legal_moves.contains(&ChessMove::CastleRight) {
            self.highlighted_castle_right = true;
        }
        if legal_moves.contains(&ChessMove::CastleLeft) {
            self.highlighted_castle_left = true;
        }
    }

    fn clear_selection(&mut self) {
        self.selected_square = None;
        self.highlighted_destinations = Vec::new();
        self.highlighted_castle_right = false;
        self.highlighted_castle_left = false;
    }

    fn confirm_move(&mut self, selected_square: Coords) {
        let mut move_to_make = ChessMove::RegularMove(Move {
            origin: selected_square,
            destination: self.cursor,
        });
        if let Some(piece_to_move) = piece_at(&self.game.board, &selected_square) {
            let row = piece_to_move.color.homerow();
            if self.cursor == (Coords { y: row, x: 2 }) && self.highlighted_castle_left {
                move_to_make = ChessMove::CastleLeft;
            } else if self.cursor == (Coords { y: row, x: 6 }) && self.highlighted_castle_right {
                move_to_make = ChessMove::CastleRight;
            }
            if let Some(epo) = self.game.history.en_passant_on {
                if piece_to_move.kind == PieceKind::Pawn && selected_square.x != self.cursor.x {
                    move_to_make = ChessMove::EnPassant(
                        Move {
                            origin: selected_square,
                            destination: self.cursor,
                        },
                        Coords {
                            x: epo.x,
                            y: epo.y + self.game.to_move.opposite().pawn_orientation(),
                        },
                    );
                }
            }

            if piece_at(&self.game.board, &selected_square)
                .is_some_and(|piece| piece.kind == PieceKind::Pawn)
                && (self.cursor.y - selected_square.y).abs() == 2
            {
                move_to_make = ChessMove::PawnSkip(Move {
                    origin: selected_square,
                    destination: self.cursor,
                });
            }
        }
        self.game.make_move(&move_to_make);

        self.clear_selection();
    }

    fn move_cursor(&mut self, delta: Coords) {
        let cursor_dest = Coords {
            x: self.cursor.x + delta.x,
            y: self.cursor.y + delta.y,
        };
        if cursor_dest.x >= 0 && cursor_dest.x < 8 && cursor_dest.y >= 0 && cursor_dest.y < 8 {
            self.cursor = cursor_dest;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

const SQUARE_SIDE: f64 = 20.0;
const OFFSET: f64 = 80.0;

fn rectangle_for_square(square: &Coords, color: Color) -> Rectangle {
    Rectangle {
        x: (SQUARE_SIDE * square.x as f64) - OFFSET,
        y: (SQUARE_SIDE * square.y as f64) - OFFSET,
        width: SQUARE_SIDE,
        height: SQUARE_SIDE,
        color,
    }
}

pub fn piece_display_name(kind: &PieceKind) -> String {
    match kind {
        PieceKind::Pawn => "Pawn".to_string(),
        PieceKind::Rook => "Rook".to_string(),
        PieceKind::Knight => "Knight".to_string(),
        PieceKind::Bishop => "Bishob".to_string(),
        PieceKind::Queen => "Queen".to_string(),
        PieceKind::King => "King".to_string(),
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Chess Time ".bold());
        let debug_ep = Line::from(format!("{:?}", self.game.history.en_passant_on)).green();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(debug_ep.centered())
            .border_set(border::THICK);

        Canvas::default()
            .block(block)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                // draw the grid and pieces
                for i in 0..8 {
                    for j in 0..8 {
                        ctx.draw(&rectangle_for_square(
                            &Coords {
                                x: j as isize,
                                y: i as isize,
                            },
                            Color::White,
                        ));
                        match &self.game.board[i][j] {
                            Some(piece) => ctx.print(
                                (SQUARE_SIDE) * j as f64 - OFFSET + SQUARE_SIDE / 4.0,
                                (SQUARE_SIDE) * i as f64 - OFFSET + SQUARE_SIDE / 2.0,
                                match piece.color {
                                    PieceColor::Black => {
                                        Line::from(piece_display_name(&piece.kind).yellow())
                                    }
                                    PieceColor::White => {
                                        Line::from(piece_display_name(&piece.kind).white())
                                    }
                                },
                            ),
                            None => {}
                        }
                    }
                }

                // highlight possible destinations
                ctx.layer();
                for square in &self.highlighted_destinations {
                    ctx.draw(&rectangle_for_square(square, Color::LightYellow));
                }

                // hightlight the selected square
                if self.selected_square.is_some() {
                    ctx.layer();
                    ctx.draw(&rectangle_for_square(
                        &self.selected_square.expect("Inside nullcheck"),
                        Color::Blue,
                    ));
                }

                // highlight the cursor
                ctx.layer();
                ctx.draw(&rectangle_for_square(&self.cursor, Color::Red));
            })
            .render(area, buf);
    }
}
