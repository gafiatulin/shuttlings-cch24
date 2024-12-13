use axum::extract::Path;
use axum::http::StatusCode;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

const WALL: char = '⬜';
const EMPTY: char = '⬛';
const COOKIE: char = '🍪';
const MILK: char = '🥛';

#[derive(PartialEq, Clone, Copy)]
pub enum Team {
    Milk,
    Cookie,
}

pub enum Terminal {
    Draw,
    Win(Team),
}

#[derive(Default)]
pub struct Board {
    cells: [[Option<Team>; 4]; 4],
    terminal: Option<Terminal>,
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in 0..4 {
            write!(f, "{}", WALL)?;
            for column in 0..4 {
                match self.cells[row][column] {
                    Some(Team::Milk) => write!(f, "{}", MILK)?,
                    Some(Team::Cookie) => write!(f, "{}", COOKIE)?,
                    None => write!(f, "{}", EMPTY)?,
                }
            }
            writeln!(f, "{}", WALL)?;
        }
        writeln!(f, "{}", WALL.to_string().repeat(6))?;
        if let Some(terminal) = &self.terminal {
            write!(
                f,
                "{}",
                match terminal {
                    Terminal::Draw => "No winner.\n",
                    Terminal::Win(Team::Milk) => "🥛 wins!\n",
                    Terminal::Win(Team::Cookie) => "🍪 wins!\n",
                }
            )?;
        }
        Ok(())
    }
}

impl Board {
    fn place(&mut self, c: usize, team: Team) -> Result<(), ()> {
        if self.terminal.is_some() {
            Err(())
        } else {
            let mut i = 3;
            let mut placed = false;
            while !placed {
                if self.cells[i][c].is_none() {
                    self.cells[i][c] = Some(team);
                    placed = true;
                } else {
                    i -= 1;
                }
            }
            if placed {
                self.check_terminal();
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn check_four(
        a: Option<Team>,
        b: Option<Team>,
        c: Option<Team>,
        d: Option<Team>,
    ) -> Option<Terminal> {
        if a == b && b == c && c == d {
            a.map(Terminal::Win)
        } else {
            None
        }
    }

    fn check_terminal(&mut self) {
        let check_rows = (0..4).find_map(|row| {
            Self::check_four(
                self.cells[row][0],
                self.cells[row][1],
                self.cells[row][2],
                self.cells[row][3],
            )
        });
        let check_columns = (0..4).find_map(|column| {
            Self::check_four(
                self.cells[0][column],
                self.cells[1][column],
                self.cells[2][column],
                self.cells[3][column],
            )
        });
        let check_diagonal1 = Self::check_four(
            self.cells[0][0],
            self.cells[1][1],
            self.cells[2][2],
            self.cells[3][3],
        );
        let check_diagonal2 = Self::check_four(
            self.cells[0][3],
            self.cells[1][2],
            self.cells[2][1],
            self.cells[3][0],
        );
        let check_filled = if self
            .cells
            .iter()
            .all(|column| column.iter().all(Option::is_some))
        {
            Some(Terminal::Draw)
        } else {
            None
        };

        self.terminal = check_columns
            .or(check_rows)
            .or(check_diagonal1)
            .or(check_diagonal2)
            .or(check_filled);
    }

    fn random(seed: &mut StdRng) -> Self {
        let mut board = Board::default();
        for r in 0..4 {
            for c in 0..4 {
                board.cells[r][c] = if seed.gen::<bool>() {
                    Some(Team::Cookie)
                } else {
                    Some(Team::Milk)
                }
            }
        }
        board.check_terminal();
        board
    }
}

pub struct State {
    board: Mutex<Board>,
    seed: Mutex<StdRng>,
}

impl Default for State {
    fn default() -> Self {
        State {
            board: Mutex::new(Board::default()),
            seed: Mutex::new(StdRng::seed_from_u64(2024)),
        }
    }
}

pub async fn board(state: Arc<State>, Path(op): Path<String>) -> (StatusCode, String) {
    match op.as_str() {
        "board" => {
            let board = state.board.lock().unwrap();
            (StatusCode::OK, format!("{}", board))
        }
        "random-board" => {
            let mut seed = state.seed.lock().unwrap();
            (StatusCode::OK, format!("{}", Board::random(&mut seed)))
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}

pub async fn game(state: Arc<State>, Path(op): Path<String>) -> (StatusCode, String) {
    match op.as_str() {
        "reset" => {
            let mut board = state.board.lock().unwrap();
            let mut seed = state.seed.lock().unwrap();
            *board = Board::default();
            *seed = StdRng::seed_from_u64(2024);
            (StatusCode::OK, format!("{}", board))
        }
        str if str.starts_with("place") => {
            match str.strip_prefix("place/").unwrap().split_once('/') {
                Some((team, column))
                    if (team == "milk" || team == "cookie") && "1234".contains(column) =>
                {
                    let mut board = state.board.lock().unwrap();
                    let team = if team == "milk" {
                        Team::Milk
                    } else {
                        Team::Cookie
                    };
                    let column = column.parse::<usize>().unwrap() - 1;
                    match board.place(column, team) {
                        Ok(_) => (StatusCode::OK, format!("{}", board)),
                        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, format!("{}", board)),
                    }
                }
                _ => (StatusCode::BAD_REQUEST, "".to_string()),
            }
        }
        _ => (StatusCode::NOT_FOUND, "".to_string()),
    }
}
