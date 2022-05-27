use anyhow::{anyhow, Result};
use std::{collections::LinkedList, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use rand::Rng;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Color,
    widgets::{Block, Borders, Widget},
    Terminal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Vector(u16, u16);

impl Vector {
    fn new(x: u16, y: u16) -> Vector {
        Vector(x, y)
    }
}

struct Dimensions {
    x: (u16, u16),
    y: (u16, u16),
}

#[derive(PartialEq, Eq)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

struct Cube {
    x: u16,
    y: u16,
}

impl Cube {
    fn new(x: u16, y: u16) -> Cube {
        Cube { x, y }
    }
}

impl Widget for Cube {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        buf.get_mut(self.x, self.y).set_bg(Color::Green);
        buf.get_mut(self.x + 1, self.y).set_bg(Color::Green);
    }
}

struct Game {
    body: LinkedList<Vector>,
    direction: Direction,
    dimensions: Dimensions,
    block: Option<Vector>,
}

impl Game {
    fn go_forward(&mut self) -> Result<()> {
        if let Some(head) = self.body.front() {
            let mut new_head = head.clone();

            match &self.direction {
                Direction::Right => self.move_vec_right(&mut new_head),
                Direction::Left => self.move_vec_left(&mut new_head),
                Direction::Up => self.move_vec_up(&mut new_head),
                Direction::Down => self.move_vec_down(&mut new_head),
            }

            if self.vec_intersects_with_body(&new_head) {
                return Err(anyhow!("game over :("));
            }

            let mut pop_back = true;

            if let Some(block) = &self.block {
                if *block == new_head {
                    self.block = None;
                    pop_back = false;
                }
            }

            self.body.push_front(new_head);

            if pop_back {
                self.body.pop_back();
            }

            Ok(())
        } else {
            Err(anyhow!("empty body"))
        }
    }

    fn vec_intersects_with_body(&self, vec: &Vector) -> bool {
        for block in &self.body {
            if *block == *vec {
                return true;
            }
        }
        return false;
    }

    fn move_vec_left(&mut self, vec: &mut Vector) {
        if vec.0 > self.dimensions.x.0 {
            vec.0 -= 1;
        } else {
            vec.0 = self.dimensions.x.1;
        }
    }

    fn move_vec_right(&mut self, vec: &mut Vector) {
        if vec.0 < self.dimensions.x.1 {
            vec.0 += 1;
        } else {
            vec.0 = self.dimensions.x.0;
        }
    }

    fn move_vec_up(&mut self, vec: &mut Vector) {
        if vec.1 > self.dimensions.y.0 {
            vec.1 -= 1;
        } else {
            vec.1 = self.dimensions.y.1;
        }
    }

    fn move_vec_down(&mut self, vec: &mut Vector) {
        if vec.1 < self.dimensions.y.1 {
            vec.1 += 1;
        } else {
            vec.1 = self.dimensions.y.0;
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut game = Game {
        body: LinkedList::from([Vector::new(5, 5)]),
        dimensions: Dimensions {
            x: (1, 20),
            y: (1, 10),
        },
        direction: Direction::Right,
        block: None,
    };
    let mut rng = rand::thread_rng();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let mut cubes: Vec<Cube> = game
            .body
            .iter()
            .map(|Vector(x, y)| Cube::new(*x, *y))
            .collect();

        if let Some(block) = &game.block {
            cubes.push(Cube::new(block.0, block.1));
        }

        let score = game.body.len();

        let size = Rect::new(0, 0, game.dimensions.x.1 + 3, game.dimensions.y.1 + 2);

        terminal.draw(|f| {
            let block = Block::default()
                .borders(Borders::empty())
                .title(format!("score: {}", score))
                .borders(Borders::ALL);

            f.render_widget(block, size);
            for cube in cubes {
                f.render_widget(cube, size);
            }
        })?;

        if let Err(_) = game.go_forward() {
            break;
        }

        if game.block.is_none() {
            game.block = Some(Vector::new(
                rng.gen_range(game.dimensions.x.0..game.dimensions.x.1),
                rng.gen_range(game.dimensions.y.0..game.dimensions.y.1),
            ));
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char(code), KeyModifiers::CONTROL) if code == 'c' || code == 'd' => {
                        break
                    }
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Up, _) if game.direction != Direction::Down => {
                        game.direction = Direction::Up
                    }
                    (KeyCode::Down, _) if game.direction != Direction::Up => {
                        game.direction = Direction::Down
                    }
                    (KeyCode::Left, _) if game.direction != Direction::Right => {
                        game.direction = Direction::Left
                    }
                    (KeyCode::Right, _) if game.direction != Direction::Left => {
                        game.direction = Direction::Right
                    }
                    _ => (),
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    return Ok(());
}
