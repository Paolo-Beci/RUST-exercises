use std::fs::OpenOptions;
use std::io::Write;
use std::io;

const bsize: usize = 20;
#[derive(Debug)]
pub struct Board {
    boats: [u8; 4],
    data: [[u8; bsize]; bsize],
}

#[derive(Debug)]
pub enum Error {
    Overlap,
    OutOfBounds,
    BoatCount,
}

pub enum Boat {
    Vertical(usize),
    Horizontal(usize),
}

impl Board {
    /** crea una board vuota con una disponibilità di navi */
    pub fn new(boats: &[u8]) -> Board {
        let mut boats_array = [0u8; 4];
        for (i, &boat) in boats.iter().enumerate() {
            if i < 4 {
                boats_array[i] = boat;
            }
        }
        
        Board {
            boats: boats_array,
            data: [[0; bsize]; bsize],
        }
    }   

    /* crea una board a partire da una stringa che rappresenta tutto
    il contenuto del file board.txt */
    pub fn from(s: String) -> Board {
        let mut lines = s.lines();

        let boats_line = lines.next().expect("File vuoto");
        let parts: Vec<u8> = boats_line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let boats: [u8; 4] = parts.try_into().expect("Devono esserci esattamente 4 numeri");

        let mut data = [[0u8; bsize]; bsize];
        for (i, line) in lines.enumerate().take(bsize) {
            for (j, ch) in line.chars().take(bsize).enumerate() {
                data[i][j] = match ch {
                    'B' => 1,
                    ' ' => 0,
                    _ => panic!("Carattere non valido: '{}'", ch),
                };
            }
        }

        Board { boats, data }
    }

    /* aggiunge la nave alla board, restituendo la nuova board se possibile */
    /* bonus: provare a *non copiare* data quando si crea e restituisce
    una nuova board con la barca, come si può fare? */
    pub fn add_boat(mut self, boat: Boat, pos: (usize, usize)) -> Result<Board, Error> {
        let len = match boat {
            Boat::Vertical(l) | Boat::Horizontal(l) => l,
        };

        if len < 1 || len > 4 || self.boats[len - 1] == 0 {
            return Err(Error::BoatCount);
        }

        let (x, y) = (pos.0 - 1, pos.1 - 1); // converti da 1-based a 0-based

        // controlla correttezza bordi
        match boat {
            Boat::Vertical(_) if x + len > bsize => return Err(Error::OutOfBounds),
            Boat::Horizontal(_) if y + len > bsize => return Err(Error::OutOfBounds),
            _ => {}
        }

        // controlla ostacoli sul posizionamento
        for i in 0..len {
            let (xi, yi) = match boat {
                Boat::Vertical(_) => (x + i, y),
                Boat::Horizontal(_) => (x, y + i),
            };

            if self.data[xi][yi] != 0 || !self.surroundings_ok(xi, yi) {
                return Err(Error::Overlap);
            }
        }

        // piazza la barca
        for i in 0..len {
            let (xi, yi) = match boat {
                Boat::Vertical(_) => (x + i, y),
                Boat::Horizontal(_) => (x, y + i),
            };
            self.data[xi][yi] = 1;
        }

        self.boats[len - 1] -= 1;
        Ok(self)
    }

    fn surroundings_ok(&self, x: usize, y: usize) -> bool {
        let start_x = x.saturating_sub(1);
        let end_x = (x + 1).min(bsize - 1);
        let start_y = y.saturating_sub(1);
        let end_y = (y + 1).min(bsize - 1);

        for i in start_x..=end_x {
            for j in start_y..=end_y {
                if self.data[i][j] != 0 {
                    return false;
                }
            }
        }
        true
    }

    /* converte la board in una stringa salvabile su file */
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        for (i, &boat_count) in self.boats.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(&boat_count.to_string());
        }
        result.push('\n');
        
        for row in &self.data {
            for &cell in row {
                if cell == 0 {
                    result.push(' ');
                } else {
                    result.push('B');
                }
            }
            result.push('\n');
        }
        
        result.pop();
        result
    }
}

pub fn main_ex2() -> Result<String, Box<dyn std::error::Error>> {
    let res = "test".to_string();
    Ok(res)
}

// ----------------- TESTS -----------------
#[cfg(test)]
mod tests {
    use super::*;
    
    const EMPTY_ROW: &str = "                    "; // 20 spazi

    #[test]
    fn test_new_board_initialization() {
        let b = Board::new(&[4, 3, 2, 1]);
        assert_eq!(b.boats, [4, 3, 2, 1]);
        for row in b.data.iter() {
            for cell in row.iter() {
                assert_eq!(*cell, 0);
            }
        }
    }

    #[test]
    fn test_board_to_string_empty() {
        let b = Board::new(&[1, 2, 3, 4]);
        let board_string = b.to_string();
        let lines: Vec<&str> = board_string.lines().collect();
        assert_eq!(lines[0], "1 2 3 4");
        for line in &lines[1..] {
            assert_eq!(*line, EMPTY_ROW);
        }
    }

    #[test]
    fn test_parse_from_string_and_back() {
        let input = format!(
            "2 2 2 2\n{}",
            std::iter::repeat(EMPTY_ROW).take(20).collect::<Vec<_>>().join("\n")
        );
        let board = Board::from(input.clone());
        assert_eq!(board.to_string(), input);
    }

    #[test]
    fn test_add_boat_success_horizontal() {
        let board = Board::new(&[1, 1, 1, 1]);
        let result = board.add_boat(Boat::Horizontal(3), (5, 5));
        assert!(result.is_ok());
        let new_board = result.unwrap();
        assert_eq!(new_board.data[4][4], 1);
        assert_eq!(new_board.data[4][5], 1);
        assert_eq!(new_board.data[4][6], 1);
        assert_eq!(new_board.boats, [1, 1, 0, 1]); // one 3-length boat used
    }

    #[test]
    fn test_add_boat_success_vertical() {
        let board = Board::new(&[1, 1, 1, 1]);
        let result = board.add_boat(Boat::Vertical(2), (3, 3));
        assert!(result.is_ok());
        let b = result.unwrap();
        assert_eq!(b.data[2][2], 1);
        assert_eq!(b.data[3][2], 1);
        assert_eq!(b.boats, [1, 0, 1, 1]); // one 2-length boat used
    }

    #[test]
    fn test_add_boat_out_of_bounds() {
        let board = Board::new(&[0, 0, 1, 0]);
        let result = board.add_boat(Boat::Horizontal(3), (20, 19));
        assert!(matches!(result, Err(Error::OutOfBounds)));
    }

    #[test]
    fn test_add_boat_overlap_error() {
        let board = Board::new(&[0, 2, 0, 0]) 
            .add_boat(Boat::Vertical(2), (10, 10))
            .unwrap();
        let result = board.add_boat(Boat::Vertical(2), (9, 9));
        assert!(matches!(result, Err(Error::Overlap)));
    }

    #[test]
    fn test_add_boat_too_many_error() {
        let board = Board::new(&[0, 0, 0, 0]);
        let result = board.add_boat(Boat::Horizontal(2), (5, 5));
        assert!(matches!(result, Err(Error::BoatCount)));
    }
}
