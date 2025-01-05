use chessticot::{piece_display_name, Coords, Game, PieceColor};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
};
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
use std::{fmt::format, io, isize};

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
}

impl App {
    pub fn init() -> App {
        App {
            exit: false,
            game: Game::start(),
            cursor: Coords { x: 0, y: 0 },
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
            KeyCode::Left => self.move_cursor(Coords { x: -1, y: 0 }),
            KeyCode::Right => self.move_cursor(Coords { x: 1, y: 0 }),
            KeyCode::Up => self.move_cursor(Coords { x: 0, y: 1 }),
            KeyCode::Down => self.move_cursor(Coords { x: 0, y: -1 }),
            _ => {}
        }
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Chess Time ".bold());
        let debug_cursor =
            Line::from(format!("X: {0} Y: {1}", self.cursor.x, self.cursor.y)).blue();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(debug_cursor.centered())
            .border_set(border::THICK);

        Canvas::default()
            .block(block)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                let square_side = 20.0;
                let offset = 80.0;
                // draw the grid and pieces
                for i in 0..8 {
                    for j in 0..8 {
                        ctx.draw(&Rectangle {
                            x: (square_side * j as f64) - offset,
                            y: (square_side * i as f64) - offset,
                            width: square_side,
                            height: square_side,
                            color: Color::White,
                        });
                        match &self.game.board[i][j] {
                            Some(piece) => ctx.print(
                                (square_side) * j as f64 - offset + square_side / 4.0,
                                (square_side) * i as f64 - offset + square_side / 2.0,
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
                // highlight the cursor
                ctx.layer();
                ctx.draw(&Rectangle {
                    x: (square_side * self.cursor.x as f64) - offset,
                    y: (square_side * self.cursor.y as f64) - offset,
                    width: square_side,
                    height: square_side,
                    color: Color::Red,
                });
            })
            .render(area, buf);
    }
}
