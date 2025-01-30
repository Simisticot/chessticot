use chessticot::{legal_moves_from_origin, ChessMove, Coords, Game, PieceColor, PieceKind};
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
use std::{collections::HashMap, io, iter::Cycle};

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
    highlighted_moves: HashMap<Coords, ChessMove>,
    promoting_to: PieceKind,
    promotion_target: Cycle<std::slice::Iter<'static, PieceKind>>,
}

impl App {
    pub fn init() -> App {
        App {
            exit: false,
            game: Game::start(),
            cursor: Coords { x: 0, y: 0 },
            selected_square: None,
            highlighted_moves: HashMap::new(),
            promoting_to: PieceKind::Queen,
            promotion_target: PieceKind::promoteable().cycle(),
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
                Some(_) => self.confirm_move(),
            },
            KeyCode::Esc => self.clear_selection(),
            KeyCode::Tab => self.cycle_promoting_to(),
            _ => {}
        }
    }

    fn cycle_promoting_to(&mut self) {
        self.promoting_to = *self.promotion_target.next().expect("should cycle");
    }

    fn select_square(&mut self) {
        self.selected_square = Some(self.cursor.clone());
        let legal_moves = legal_moves_from_origin(&self.cursor, &self.game.current_position);
        legal_moves.iter().for_each(|chess_move| {
            let starting_row = self.game.current_position.to_move.homerow();
            match chess_move {
                ChessMove::RegularMove(coordinates) => self
                    .highlighted_moves
                    .insert(coordinates.destination, chess_move.clone()),
                ChessMove::PawnSkip(coordinates) => self
                    .highlighted_moves
                    .insert(coordinates.destination, chess_move.clone()),
                ChessMove::CastleLeft => self.highlighted_moves.insert(
                    Coords {
                        y: starting_row,
                        x: 2,
                    },
                    chess_move.clone(),
                ),
                ChessMove::CastleRight => self.highlighted_moves.insert(
                    Coords {
                        y: starting_row,
                        x: 6,
                    },
                    chess_move.clone(),
                ),
                ChessMove::EnPassant(movement, _) => self
                    .highlighted_moves
                    .insert(movement.destination, chess_move.clone()),
                ChessMove::Promotion(movement, _) => self
                    .highlighted_moves
                    .insert(movement.destination, chess_move.clone()),
            };
        });
    }

    fn clear_selection(&mut self) {
        self.selected_square = None;
        self.highlighted_moves = HashMap::new();
    }

    fn confirm_move(&mut self) {
        if let Some(move_to_make) = self.highlighted_moves.get(&self.cursor) {
            if let ChessMove::Promotion(movement, _) = move_to_make {
                self.game.make_move(&ChessMove::Promotion(
                    movement.clone(),
                    self.promoting_to.clone(),
                ));
            } else {
                self.game.make_move(&move_to_make);
            }
        }

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
        let promoting_to_label = Line::from(format!("promoting to {:?}", self.promoting_to))
            .centered()
            .green();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(promoting_to_label)
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
                        match &self.game.current_position.board[i][j] {
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
                for square in self.highlighted_moves.keys() {
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
