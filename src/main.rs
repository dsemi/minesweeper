mod game;

use game::Game;
use std::collections::HashSet;
use std::ops::Sub;
use Sq::*;
use Square::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Square {
    Unknown,
    Mine,
    Blank,
    Adj(u8),
}

impl std::convert::TryFrom<u8> for Square {
    type Error = String;

    fn try_from(value: u8) -> Result<Square, Self::Error> {
        match value {
            0 => Ok(Blank),
            n if n < 9 => Ok(Adj(n)),
            9 => Ok(Mine),
            _ => Err(format!("Invalid value: {}", value)),
        }
    }
}

struct Player {
    game: Game,
    groups: Vec<Vec<Sq>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Sq {
    Unprocessed,
    Grps(Vec<Group>),
    Done,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Group {
    mines: usize,
    unknowns: HashSet<(usize, usize)>,
}

impl Sub for &Group {
    type Output = Group;

    fn sub(self, rhs: &Group) -> Self::Output {
        Group {
            mines: self.mines - rhs.mines,
            unknowns: self.unknowns.difference(&rhs.unknowns).copied().collect(),
        }
    }
}

impl Player {
    fn from_game(mut game: Game) -> Self {
        // First click for known non-mine position.
        game.click(0, 6);
        Self {
            groups: vec![vec![Unprocessed; game.cols]; game.rows],
            game,
        }
    }

    fn click_all<'a, I>(&mut self, iter: I) -> Result<(), String>
    where
        I: Iterator<Item = &'a (usize, usize)>,
    {
        for &(r, c) in iter {
            if !self.game.click(r, c) {
                return Err(format!("Clicked on a mine at ({}, {})", r, c));
            }
        }
        Ok(())
    }

    fn mark_all<'a, I>(&mut self, iter: I) -> Result<(), String>
    where
        I: Iterator<Item = &'a (usize, usize)>,
    {
        for &(r, c) in iter {
            if !self.game.mark_mine(r, c) {
                return Err(format!("Marked a non-mine as a mine at ({}, {})", r, c));
            }
        }
        Ok(())
    }

    fn play(&mut self) -> Result<(), String> {
        let mut changed = true;
        while std::mem::take(&mut changed) {
            for r in 0..self.game.rows {
                for c in 0..self.game.cols {
                    if self.groups[r][c] == Done {
                        continue;
                    }
                    if let Adj(n) = self.game.visible_grid[r][c] {
                        let adj = self.game.adj(r, c);
                        let mut known_mines = 0;
                        let mut unknowns = HashSet::new();
                        for &(r, c) in &adj {
                            match self.game.visible_grid[r][c] {
                                Mine => known_mines += 1,
                                Unknown => {
                                    unknowns.insert((r, c));
                                }
                                _ => (),
                            }
                        }
                        let group = Group {
                            mines: n as usize - known_mines,
                            unknowns,
                        };
                        if group.mines == 0 && group.unknowns.len() > 0 {
                            self.click_all(group.unknowns.iter())?;
                            changed = true;
                            self.groups[r][c] = Done;
                            continue;
                        }
                        if group.mines == group.unknowns.len() && group.unknowns.len() > 0 {
                            self.mark_all(group.unknowns.iter())?;
                            changed = true;
                            self.groups[r][c] = Done;
                            continue;
                        }
                        let adj_groups = self
                            .game
                            .adj2(r, c)
                            .into_iter()
                            .filter_map(|(r, c)| match &self.groups[r][c] {
                                Grps(grps) => Some(grps.clone()),
                                _ => None,
                            })
                            .flatten()
                            .collect::<Vec<_>>();
                        let subs = adj_groups
                            .clone()
                            .into_iter()
                            .filter(|grp| grp.unknowns.is_subset(&group.unknowns))
                            .collect::<Vec<_>>();
                        match &subs[..] {
                            [sub] => {
                                let diff_group = &group - &sub;
                                if diff_group.mines > 0
                                    && diff_group.unknowns.len() == diff_group.mines
                                {
                                    self.mark_all(diff_group.unknowns.iter())?;
                                    changed = true;
                                    continue;
                                } else if diff_group.mines == 0 && diff_group.unknowns.len() > 0 {
                                    self.click_all(diff_group.unknowns.iter())?;
                                    changed = true;
                                    continue;
                                }
                            }
                            [sub1, sub2] if sub1.mines > 0 && sub2.mines > 0 => {
                                let inter = &sub1.unknowns & &sub2.unknowns;
                                if group.mines == 1 && inter.len() == 1 {
                                    self.mark_all(inter.iter())?;
                                    changed = true;
                                    continue;
                                }
                            }
                            _ => (),
                        }
                        let mut grps = vec![group.clone()];
                        for grp in adj_groups {
                            if grp.unknowns.is_subset(&group.unknowns) {
                                let diff_group = &group - &grp;
                                if diff_group.mines > 0
                                    && diff_group.mines == diff_group.unknowns.len()
                                {
                                    self.mark_all(diff_group.unknowns.iter())?;
                                    changed = true;
                                    continue;
                                } else if diff_group.mines == 0 && diff_group.unknowns.len() > 0 {
                                    self.click_all(diff_group.unknowns.iter())?;
                                    changed = true;
                                    continue;
                                } else if diff_group.unknowns.len() > 0 {
                                    grps.push(diff_group);
                                }
                            } else {
                                let inter = &group.unknowns & &grp.unknowns;
                                let non_inter_len = grp.unknowns.len() - inter.len();
                                let lo = grp.mines.saturating_sub(non_inter_len);
                                let hi = grp.mines.max(inter.len());
                                let mut poss = vec![];
                                for m in lo..=hi {
                                    if m <= group.mines
                                        && group.mines - m <= group.unknowns.len() - inter.len()
                                    {
                                        poss.push(
                                            &group
                                                - &Group {
                                                    mines: m,
                                                    unknowns: inter.clone(),
                                                },
                                        );
                                    }
                                }
                                match &poss[..] {
                                    [grp] if grp.unknowns.len() > 0 => {
                                        if grp.mines > 0 && grp.mines == grp.unknowns.len() {
                                            self.mark_all(grp.unknowns.iter())?;
                                            changed = true;
                                        } else if grp.mines == 0 && grp.unknowns.len() > 0 {
                                            self.click_all(grp.unknowns.iter())?;
                                            changed = true;
                                        }
                                        continue;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        match &self.groups[r][c] {
                            Unprocessed => {
                                self.groups[r][c] = Grps(grps);
                                changed = true;
                            }
                            Grps(gs) if &grps != gs => {
                                self.groups[r][c] = Grps(grps);
                                changed = true;
                            }
                            _ => (),
                        }
                    }
                }
            }
        }

        println!("{}", self.game);
        Ok(())
    }
}

fn main() {
    #[rustfmt::skip]
    let sample = vec![
        vec![1, 1, 2, 9, 2, 1, 0, 0, 1, 9, 1, 0, 1, 9, 2, 9, 1, 2, 9, 2, 0, 0, 0, 0, 1, 1, 1, 2, 9, 2],
        vec![1, 9, 2, 2, 9, 1, 0, 0, 1, 1, 1, 0, 2, 2, 3, 2, 2, 3, 9, 2, 1, 2, 3, 2, 3, 9, 3, 3, 9, 2],
        vec![1, 1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 1, 9, 1, 1, 9, 2, 2, 2, 2, 9, 9, 9, 4, 9, 9, 2, 1, 1],
        vec![0, 0, 1, 9, 1, 0, 0, 1, 1, 1, 0, 0, 2, 3, 3, 2, 1, 1, 1, 9, 2, 2, 4, 5, 9, 5, 3, 1, 0, 0],
        vec![0, 0, 1, 1, 1, 1, 2, 3, 9, 2, 1, 1, 1, 9, 9, 1, 0, 0, 1, 1, 1, 0, 1, 9, 9, 9, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 9, 9, 3, 3, 9, 1, 1, 3, 4, 3, 2, 1, 1, 0, 0, 1, 2, 3, 4, 3, 2, 1, 1, 1],
        vec![0, 0, 0, 0, 1, 2, 4, 3, 3, 9, 2, 1, 0, 1, 9, 9, 3, 9, 1, 1, 1, 3, 9, 4, 3, 9, 2, 2, 9, 1],
        vec![0, 0, 0, 0, 1, 9, 2, 9, 3, 2, 1, 0, 1, 2, 4, 4, 9, 2, 1, 2, 9, 5, 9, 9, 9, 5, 9, 4, 2, 1],
        vec![0, 0, 1, 2, 3, 2, 2, 2, 9, 2, 1, 1, 2, 9, 3, 9, 2, 1, 0, 2, 9, 5, 9, 6, 4, 9, 9, 9, 2, 1],
        vec![0, 0, 1, 9, 9, 1, 0, 1, 2, 3, 9, 1, 2, 9, 3, 1, 2, 2, 2, 2, 1, 3, 9, 3, 9, 3, 3, 2, 3, 9],
        vec![2, 2, 2, 3, 3, 2, 0, 0, 1, 9, 3, 3, 2, 2, 1, 0, 2, 9, 9, 1, 0, 1, 2, 3, 2, 2, 1, 2, 3, 9],
        vec![9, 9, 1, 1, 9, 2, 1, 2, 3, 4, 9, 2, 9, 2, 2, 3, 4, 9, 3, 1, 1, 2, 3, 9, 1, 1, 9, 3, 9, 3],
        vec![2, 2, 1, 1, 2, 3, 9, 3, 9, 9, 2, 2, 1, 2, 9, 9, 9, 2, 2, 2, 4, 9, 9, 2, 2, 2, 3, 9, 4, 9],
        vec![0, 0, 0, 0, 1, 9, 3, 5, 9, 4, 1, 1, 2, 4, 5, 6, 4, 2, 2, 9, 9, 9, 3, 1, 1, 9, 3, 3, 9, 2],
        vec![1, 1, 1, 0, 2, 2, 3, 9, 9, 2, 0, 1, 9, 9, 9, 9, 9, 1, 3, 9, 9, 3, 1, 0, 1, 3, 9, 4, 3, 3],
        vec![1, 9, 2, 1, 3, 9, 3, 2, 2, 1, 0, 1, 3, 4, 4, 3, 3, 2, 4, 9, 4, 1, 0, 1, 1, 4, 9, 4, 9, 9],
        vec![1, 1, 2, 9, 3, 9, 2, 1, 1, 2, 1, 1, 1, 9, 2, 1, 2, 9, 5, 9, 3, 0, 0, 1, 9, 3, 9, 3, 2, 2],
        vec![0, 0, 1, 1, 2, 1, 1, 1, 9, 2, 9, 2, 2, 3, 9, 1, 3, 9, 5, 9, 2, 1, 2, 3, 3, 3, 2, 1, 0, 0],
        vec![0, 0, 1, 1, 1, 0, 0, 2, 3, 4, 2, 2, 9, 2, 2, 2, 3, 9, 3, 1, 1, 2, 9, 9, 2, 9, 3, 2, 1, 0],
        vec![0, 0, 1, 9, 1, 0, 0, 1, 9, 9, 1, 1, 1, 1, 1, 9, 2, 1, 1, 0, 0, 2, 9, 3, 2, 2, 9, 9, 1, 0],
    ];

    let mut player = Player::from_game(Game::from_sol(sample));
    player.play().unwrap();
}
