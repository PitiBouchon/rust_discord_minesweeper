#![feature(let_chains)]

use rand::random;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MinesweeperCellType {
    Hidden,
    Bomb,
    BombExploded,
    Found(u8),
}

#[derive(Debug)]
pub struct MinesweeperGrid(pub Vec<Vec<MinesweeperCellType>>);

impl MinesweeperGrid {
    /// Create a new MinesweeperGrid
    ///
    /// Example :
    /// ```rust
    /// # use minesweeper::MinesweeperGrid;
    /// let mut grid = MinesweeperGrid::new(10, 10, 0.3);
    /// ```
    pub fn new(width: usize, height: usize, bomb_probability: f64) -> Self {
        Self(
            (0..width)
                .map(|_| {
                    (0..height)
                        .map(|_| {
                            if random::<f64>() > bomb_probability {
                                MinesweeperCellType::Hidden
                            } else {
                                MinesweeperCellType::Bomb
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
        )
    }

    pub fn to_console_string(&self) -> String {
        self.0
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|cell| match cell {
                        MinesweeperCellType::Hidden
                        | MinesweeperCellType::Bomb
                        | MinesweeperCellType::BombExploded => "_".to_string(),
                        MinesweeperCellType::Found(n) => n.to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_discord_string(&self) -> String {
        self.0
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|cell| match cell {
                        MinesweeperCellType::Hidden => "🟫".to_string(),
                        MinesweeperCellType::Bomb => "💣".to_string(),
                        MinesweeperCellType::BombExploded => "🧨".to_string(),
                        MinesweeperCellType::Found(n) => match n {
                            0 => "0️⃣".to_string(),
                            1 => "1️⃣".to_string(),
                            2 => "2️⃣".to_string(),
                            3 => "3️⃣".to_string(),
                            4 => "4️⃣".to_string(),
                            5 => "5️⃣".to_string(),
                            6 => "6️⃣".to_string(),
                            7 => "7️⃣".to_string(),
                            8 => "8️⃣".to_string(),
                            _ => unreachable!(),
                        },
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn reveal_zone(&mut self, xpos: usize, ypos: usize) -> usize {
        let mut revealed_cell = 0;

        let neighbours = self.get_neighbours(xpos, ypos);

        let neighbours_bomb = neighbours
            .iter()
            .filter(|(cell, _)| *cell == MinesweeperCellType::Bomb)
            .count() as u8;

        if let Some(cell) = self.get_mut_cell(xpos, ypos) && *cell == MinesweeperCellType::Hidden {
            *cell = MinesweeperCellType::Found(neighbours_bomb);
            revealed_cell += 1;

            if neighbours_bomb == 0 {
                for (neighbour_cell, (xpos, ypos)) in neighbours {
                    if neighbour_cell == MinesweeperCellType::Hidden {
                        revealed_cell += self.reveal_zone(xpos, ypos);
                    }
                }
            }
        }

        revealed_cell
    }

    /// Return the number of cells revealed
    /// Discover from a position of a grid and return the number of cells revealed
    ///
    /// Example :
    /// ```rust
    /// # use minesweeper::MinesweeperGrid;
    /// # let mut grid = MinesweeperGrid::new(10, 10, 0.3);
    /// let cell_revealed = grid.discover(5, 5);
    /// ```
    /// Return `None` if the position is :
    /// - A bomb
    /// - Already revealed
    /// - Invalid
    pub fn discover(&mut self, xpos: usize, ypos: usize) -> Option<usize> {
        let first_attempt = !self.0.iter().any(|column| {
            column
                .iter()
                .any(|cell| matches!(*cell, MinesweeperCellType::Found(_)))
        });

        let cell = self.get_cell(xpos, ypos)?;
        match cell {
            MinesweeperCellType::Hidden => Some(self.reveal_zone(xpos, ypos)),
            MinesweeperCellType::Bomb => {
                if first_attempt {
                    *self.get_mut_cell(xpos, ypos)? = MinesweeperCellType::Hidden;
                    Some(self.reveal_zone(xpos, ypos))
                } else {
                    *self.get_mut_cell(xpos, ypos)? = MinesweeperCellType::BombExploded;
                    None
                }
            }
            MinesweeperCellType::BombExploded => None,
            MinesweeperCellType::Found(_) => None,
        }
    }

    fn get_neighbours(
        &self,
        xpos: usize,
        ypos: usize,
    ) -> Vec<(MinesweeperCellType, (usize, usize))> {
        let mut v = Vec::with_capacity(8);
        for x in -1..=1 {
            for y in -1..=1 {
                if (x != 0 || y != 0) &&
                    let Some(neighbour_xpos) = xpos.checked_add_signed(x) &&
                    let Some(neighbour_ypos) = ypos.checked_add_signed(y)
                {
                    if let Some(cell) =
                        self.get_cell(neighbour_xpos, neighbour_ypos)
                    {
                        v.push((*cell, (neighbour_xpos, neighbour_ypos)));
                    }
                }
            }
        }

        v
    }

    fn get_cell(&self, xpos: usize, ypos: usize) -> Option<&MinesweeperCellType> {
        self.0.get(xpos)?.get(ypos)
    }

    fn get_mut_cell(&mut self, xpos: usize, ypos: usize) -> Option<&mut MinesweeperCellType> {
        self.0.get_mut(xpos)?.get_mut(ypos)
    }
}
