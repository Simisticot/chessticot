use chessticot::{
    BasicEvaluationPlayer, BetterEvaluationPlayer, ChessMove, Coords, FirstMovePlayer, Game,
    PieceColor, PieceKind, Player, Position, RandomCapturePrioPlayer, RandomPlayer,
};
use core::panic;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Paragraph, Widget, Wrap,
    },
    DefaultTerminal, Frame,
};
use std::{
    collections::HashMap,
    env::{self},
    io,
    iter::Cycle,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let starting_position = if args.len() > 1 {
        Some(Position::from_fen(&args[1]))
    } else {
        None
    };
    let mut terminal = ratatui::init();
    let app_result = App::init(starting_position).run(&mut terminal);
    ratatui::restore();
    app_result
}

enum Screen {
    MainMenu,
    Game,
    Result,
}

#[derive(Clone)]
enum AvailableEngine {
    First,
    Random,
    PrioritizeCapture,
    BasicEval,
    BetterEval,
}

impl AvailableEngine {
    fn in_order() -> Cycle<std::array::IntoIter<AvailableEngine, 5>> {
        [
            AvailableEngine::First,
            AvailableEngine::Random,
            AvailableEngine::PrioritizeCapture,
            AvailableEngine::BasicEval,
            AvailableEngine::BetterEval,
        ]
        .into_iter()
        .cycle()
    }
    fn get_engine(&self) -> Box<dyn Player> {
        match self {
            AvailableEngine::First => Box::new(FirstMovePlayer {}),
            AvailableEngine::Random => Box::new(RandomPlayer {}),
            AvailableEngine::PrioritizeCapture => Box::new(RandomCapturePrioPlayer {}),
            AvailableEngine::BasicEval => Box::new(BasicEvaluationPlayer {}),
            AvailableEngine::BetterEval => Box::new(BetterEvaluationPlayer {}),
        }
    }
}

pub struct App {
    exit: bool,
    game: Game,
    cursor: Coords,
    selected_square: Option<Coords>,
    highlighted_moves: HashMap<Coords, ChessMove>,
    promoting_to: PieceKind,
    promotion_target: Cycle<std::slice::Iter<'static, PieceKind>>,
    selectable_colors: Cycle<std::array::IntoIter<PieceColor, 2>>,
    selected_color: PieceColor,
    current_screen: Screen,
    selected_engine: Box<dyn Player>,
    available_engines: Cycle<std::array::IntoIter<AvailableEngine, 5>>,
    evalutation: isize,
}

impl App {
    pub fn init(starting_position: Option<Position>) -> App {
        let mut game = Game::start();
        if let Some(position) = starting_position {
            game = Game::from_starting_position(position);
        }
        App {
            exit: false,
            game,
            cursor: Coords { x: 0, y: 0 },
            selected_square: None,
            highlighted_moves: HashMap::new(),
            promoting_to: PieceKind::Queen,
            promotion_target: PieceKind::promoteable().cycle(),
            selectable_colors: PieceColor::both().cycle(),
            selected_color: PieceColor::Black,
            current_screen: Screen::MainMenu,
            selected_engine: Box::new(BetterEvaluationPlayer {}),
            available_engines: AvailableEngine::in_order(),
            evalutation: 0,
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
        let _ = match self.current_screen {
            Screen::MainMenu => self.handle_events_main_menu(),
            Screen::Game => self.handle_events_game(),
            Screen::Result => self.handle_events_result(),
        };
        Ok(())
    }
    fn handle_events_result(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }
    fn handle_events_game(&mut self) -> io::Result<()> {
        if self.game.checkmated.is_some() || self.game.stalemate {
            self.current_screen = Screen::Result;
            return Ok(());
        }
        if self.game.current_position.to_move == self.selected_color {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event_game(key_event)
                }
                _ => {}
            };
        } else {
            let offered_move = self.selected_engine.offer_move(&self.game.current_position);
            if self.game.current_position.is_move_legal(&offered_move) {
                self.game.make_move(&offered_move);
            } else {
                panic!("engine offered illegal move");
            }
        }

        Ok(())
    }

    fn handle_events_main_menu(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Right | KeyCode::Left => {
                        self.selected_color = self.selectable_colors.next().unwrap()
                    }
                    KeyCode::Enter => self.current_screen = Screen::Game,
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('e') => {
                        self.selected_engine = self.available_engines.next().unwrap().get_engine()
                    }
                    _ => (),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event_game(&mut self, key_event: KeyEvent) {
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
        let legal_moves = self
            .game
            .current_position
            .legal_moves_from_origin(&self.cursor);
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
                self.evalutation = self.selected_engine.evalutate(
                    &self
                        .game
                        .current_position
                        .color_to_move(self.selected_color.opposite()),
                );
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
        match self.current_screen {
            Screen::MainMenu => {
                let color_selection_title = Line::from(" Choose color ".bold());
                let color_selection_text = Line::from(format!(
                    "Play as {} (pick with arrow keys)",
                    self.selected_color
                ));
                let engine_selection_text = Line::from(format!(
                    "Play against {} (change with 'e')",
                    self.selected_engine
                ));
                Paragraph::new(Text::from(vec![
                    color_selection_text.white().centered(),
                    engine_selection_text.white().centered(),
                ]))
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .title(color_selection_title.clone().centered())
                        .border_set(border::THICK),
                )
                .render(area, buf);
            }
            Screen::Result => {
                let result_title = Line::from(" Game Over ".bold());
                let result_card = Block::bordered()
                    .title(result_title.centered())
                    .border_set(border::THICK);
                if let Some(color) = self.game.checkmated {
                    let result_text =
                        Line::from(format!(" {} wins by checkmate !", color.opposite()));
                    Paragraph::new(result_text.white().centered())
                        .wrap(Wrap { trim: true })
                        .block(result_card)
                        .render(area, buf);
                } else {
                    let result_text = Line::from("Stalemate !");
                    Paragraph::new(result_text.white().centered())
                        .wrap(Wrap { trim: true })
                        .block(result_card)
                        .render(area, buf);
                }
            }
            Screen::Game => {
                let title = Line::from(" Chess Time ".bold());
                let promoting_to_label =
                    Line::from(format!("promoting to {:?}", self.promoting_to))
                        .centered()
                        .green();
                let engine_evaluation_label =
                    Line::from(format!("Evaluation: {}", self.evalutation));
                let block = Block::bordered()
                    .title(title.centered())
                    .title_bottom(engine_evaluation_label.left_aligned())
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
    }
}
