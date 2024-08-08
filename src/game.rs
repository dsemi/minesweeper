use crate::Square;
use crate::Square::*;
use std::fmt;

pub struct Game {
    pub rows: usize,
    pub cols: usize,
    pub visible_grid: Vec<Vec<Square>>,
    solution: Vec<Vec<Square>>,
}

impl Game {
    pub fn from_sol(solution: Vec<Vec<u8>>) -> Self {
        let rows = solution.len();
        let cols = solution[0].len();
        Self {
            rows,
            cols,
            visible_grid: vec![vec![Unknown; cols]; rows],
            solution: solution
                .into_iter()
                .map(|row| row.into_iter().map(|v| v.try_into().unwrap()).collect())
                .collect(),
        }
    }

    pub fn click(&mut self, r: usize, c: usize) -> bool {
        self.visible_grid[r][c] = self.solution[r][c];
        match self.solution[r][c] {
            Mine => return false,
            Blank => self.adj(r, c).into_iter().for_each(|(r, c)| {
                if self.visible_grid.get(r).and_then(|row| row.get(c)) == Some(&Unknown) {
                    self.click(r, c);
                }
            }),
            _ => (),
        }
        true
    }

    pub fn mark_mine(&mut self, r: usize, c: usize) -> bool {
        if self.solution[r][c] == Mine {
            self.visible_grid[r][c] = Mine;
            return true;
        }
        false
    }

    pub fn adj(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        [
            (r.wrapping_sub(1), c.wrapping_sub(1)),
            (r.wrapping_sub(1), c),
            (r.wrapping_sub(1), c.wrapping_add(1)),
            (r, c.wrapping_sub(1)),
            (r, c.wrapping_add(1)),
            (r.wrapping_add(1), c.wrapping_sub(1)),
            (r.wrapping_add(1), c),
            (r.wrapping_add(1), c.wrapping_add(1)),
        ]
        .into_iter()
        .filter(|&(r, c)| r < self.rows && c < self.cols)
        .collect()
    }

    pub fn adj2(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        [
            (r.wrapping_sub(2), c.wrapping_sub(2)),
            (r.wrapping_sub(2), c.wrapping_sub(1)),
            (r.wrapping_sub(2), c),
            (r.wrapping_sub(2), c.wrapping_add(1)),
            (r.wrapping_sub(2), c.wrapping_add(2)),
            (r.wrapping_sub(1), c.wrapping_sub(2)),
            (r.wrapping_sub(1), c.wrapping_sub(1)),
            (r.wrapping_sub(1), c),
            (r.wrapping_sub(1), c.wrapping_add(1)),
            (r.wrapping_sub(1), c.wrapping_add(2)),
            (r, c.wrapping_sub(2)),
            (r, c.wrapping_sub(1)),
            (r, c.wrapping_add(1)),
            (r, c.wrapping_add(2)),
            (r.wrapping_add(1), c.wrapping_sub(2)),
            (r.wrapping_add(1), c.wrapping_sub(1)),
            (r.wrapping_add(1), c),
            (r.wrapping_add(1), c.wrapping_add(1)),
            (r.wrapping_add(1), c.wrapping_add(2)),
            (r.wrapping_add(2), c.wrapping_sub(2)),
            (r.wrapping_add(2), c.wrapping_sub(1)),
            (r.wrapping_add(2), c),
            (r.wrapping_add(2), c.wrapping_add(1)),
            (r.wrapping_add(2), c.wrapping_add(2)),
        ]
        .into_iter()
        .filter(|&(r, c)| r < self.rows && c < self.cols)
        .collect()
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.visible_grid.iter() {
            for sq in row.iter() {
                let c = match sq {
                    Unknown => "#",
                    Mine => "X",
                    Blank => " ",
                    Adj(1) => "\x1b[34m1\x1b[0m",
                    Adj(2) => "\x1b[32m2\x1b[0m",
                    Adj(3) => "\x1b[31m3\x1b[0m",
                    Adj(4) => "\x1b[37m4\x1b[0m",
                    Adj(5) => "\x1b[33m5\x1b[0m",
                    Adj(6) => "\x1b[36m6\x1b[0m",
                    Adj(7) => "7",
                    Adj(8) => "8",
                    _ => unreachable!(),
                };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
