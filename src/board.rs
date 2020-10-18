use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub struct Board {
    pub w_p_bb: u64,
    pub w_n_bb: u64,
    pub w_b_bb: u64,
    pub w_r_bb: u64,
    pub w_q_bb: u64,
    pub w_k_bb: u64,
    pub b_p_bb: u64,
    pub b_n_bb: u64,
    pub b_b_bb: u64,
    pub b_r_bb: u64,
    pub b_q_bb: u64,
    pub b_k_bb: u64,
    pub is_w_move: bool,
    pub is_w_castle: bool,
    pub is_w_q_castle: bool,
    pub is_b_castle: bool,
    pub is_b_q_castle: bool,
    pub en_passent: Option<u64>,
    pub halfmove_clock: u32,
    pub fullmove_clock: u32,
}

#[derive(PartialEq, Eq)]
enum PieceType {
    WP,
    WN,
    WB,
    WR,
    WQ,
    WK,
    BP,
    BN,
    BB,
    BR,
    BQ,
    BK,
}

impl Board {
    pub fn new(fen: &str) -> Board {
        let fen_tokens: Vec<String> = fen
            .split_ascii_whitespace()
            .map(|x| String::from(x))
            .collect();

        if fen_tokens.len() != 6 {
            panic!("Invalid fen: {}", fen);
        }

        let mut w_p_bb = 0;
        let mut w_n_bb = 0;
        let mut w_b_bb = 0;
        let mut w_r_bb = 0;
        let mut w_q_bb = 0;
        let mut w_k_bb = 0;
        let mut b_p_bb = 0;
        let mut b_n_bb = 0;
        let mut b_b_bb = 0;
        let mut b_r_bb = 0;
        let mut b_q_bb = 0;
        let mut b_k_bb = 0;

        let mut row: u32 = 7;
        let mut col: u32 = 0;
        for c in fen_tokens[0].chars() {
            match c {
                'p' => b_p_bb |= 0x1 << (row * 8 + col),
                'n' => b_n_bb |= 0x1 << (row * 8 + col),
                'b' => b_b_bb |= 0x1 << (row * 8 + col),
                'r' => b_r_bb |= 0x1 << (row * 8 + col),
                'q' => b_q_bb |= 0x1 << (row * 8 + col),
                'k' => b_k_bb |= 0x1 << (row * 8 + col),
                'P' => w_p_bb |= 0x1 << (row * 8 + col),
                'N' => w_n_bb |= 0x1 << (row * 8 + col),
                'B' => w_b_bb |= 0x1 << (row * 8 + col),
                'R' => w_r_bb |= 0x1 << (row * 8 + col),
                'Q' => w_q_bb |= 0x1 << (row * 8 + col),
                'K' => w_k_bb |= 0x1 << (row * 8 + col),
                n @ '1'..='8' => col += n.to_digit(10).unwrap() - 1,
                '/' => {
                    row -= 1;
                    col = 0;
                    continue;
                }
                _ => panic!("Invalid character in fen board: {}", c),
            }
            col += 1;
        }

        let is_white_move = if fen_tokens[1] == "w" { true } else { false };

        let mut w_castle = false;
        let mut w_q_castle = false;
        let mut b_castle = false;
        let mut b_q_castle = false;

        for c in fen_tokens[2].chars() {
            match c {
                'K' => w_castle = true,
                'Q' => w_q_castle = true,
                'k' => b_castle = true,
                'q' => b_q_castle = true,
                '-' => (),
                _ => panic!("Invalid character in fen castling rights: {}", c),
            }
        }

        let en_p_str = &fen_tokens[3];
        let en_passent = if en_p_str.chars().count() == 2 {
            let mut iter = en_p_str.chars();
            let col = iter.next().unwrap();
            let mut row: u32 = iter.next().unwrap().to_digit(10).unwrap();
            row -= 1;
            let col = match col {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
                _ => panic!("Invalid character in fen en passent: {}", col),
            };

            Some(0x1 << (row * 8 + col))
        } else {
            None
        };

        let halfmove = fen_tokens[4].parse().unwrap();
        let fullmove = fen_tokens[5].parse().unwrap();

        Board {
            w_p_bb: w_p_bb,
            w_n_bb: w_n_bb,
            w_b_bb: w_b_bb,
            w_r_bb: w_r_bb,
            w_q_bb: w_q_bb,
            w_k_bb: w_k_bb,
            b_p_bb: b_p_bb,
            b_n_bb: b_n_bb,
            b_b_bb: b_b_bb,
            b_r_bb: b_r_bb,
            b_q_bb: b_q_bb,
            b_k_bb: b_k_bb,
            is_w_move: is_white_move,
            is_w_castle: w_castle,
            is_w_q_castle: w_q_castle,
            is_b_castle: b_castle,
            is_b_q_castle: b_q_castle,
            en_passent: en_passent,
            halfmove_clock: halfmove,
            fullmove_clock: fullmove,
        }
    }

    pub fn do_move(&mut self, mov: &str) {
        let mut iter = mov.chars();
        let from_col = iter.next().unwrap();
        let from_row = iter.next().unwrap().to_digit(10).unwrap() - 1;
        let to_col = iter.next().unwrap();
        let to_row = iter.next().unwrap().to_digit(10).unwrap() - 1;

        let from_col = match from_col {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Invalid moves command: {}", mov),
        };
        let to_col = match to_col {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Invalid moves command: {}", mov),
        };

        let from_ind = from_row * 8 + from_col;
        let to_ind = to_row * 8 + to_col;

        // Find the bitboard responsible for the move
        let from_pos = 0x1 << from_ind;
        let to_pos = 0x1 << to_ind;
        let from_pt = if self.w_p_bb & from_pos > 0 {
            PieceType::WP
        } else if self.w_n_bb & from_pos > 0 {
            PieceType::WN
        } else if self.w_b_bb & from_pos > 0 {
            PieceType::WB
        } else if self.w_r_bb & from_pos > 0 {
            PieceType::WR
        } else if self.w_q_bb & from_pos > 0 {
            PieceType::WQ
        } else if self.w_k_bb & from_pos > 0 {
            PieceType::WK
        } else if self.b_p_bb & from_pos > 0 {
            PieceType::BP
        } else if self.b_n_bb & from_pos > 0 {
            PieceType::BN
        } else if self.b_b_bb & from_pos > 0 {
            PieceType::BB
        } else if self.b_r_bb & from_pos > 0 {
            PieceType::BR
        } else if self.b_q_bb & from_pos > 0 {
            PieceType::BQ
        } else if self.b_k_bb & from_pos > 0 {
            PieceType::BK
        } else {
            panic!("Invalid moves command: {}", mov);
        };
        let to_pt = if self.w_p_bb & to_pos > 0 {
            Some(PieceType::WP)
        } else if self.w_n_bb & to_pos > 0 {
            Some(PieceType::WN)
        } else if self.w_b_bb & to_pos > 0 {
            Some(PieceType::WB)
        } else if self.w_r_bb & to_pos > 0 {
            Some(PieceType::WR)
        } else if self.w_q_bb & to_pos > 0 {
            Some(PieceType::WQ)
        } else if self.w_k_bb & to_pos > 0 {
            Some(PieceType::WK)
        } else if self.b_p_bb & to_pos > 0 {
            Some(PieceType::BP)
        } else if self.b_n_bb & to_pos > 0 {
            Some(PieceType::BN)
        } else if self.b_b_bb & to_pos > 0 {
            Some(PieceType::BB)
        } else if self.b_r_bb & to_pos > 0 {
            Some(PieceType::BR)
        } else if self.b_q_bb & to_pos > 0 {
            Some(PieceType::BQ)
        } else if self.b_k_bb & to_pos > 0 {
            Some(PieceType::BK)
        } else {
            None
        };

        // Set the side to move
        match from_pt {
            PieceType::WP
            | PieceType::WN
            | PieceType::WB
            | PieceType::WR
            | PieceType::WQ
            | PieceType::WK => self.is_w_move = false,
            _ => self.is_w_move = true,
        }

        //Increment move counters
        self.halfmove_clock += 1;
        if self.is_w_move {
            self.fullmove_clock += 1;
        }

        // White en_passent valid
        if from_pt == PieceType::WP && from_row == 1 && to_row == 3 {
            self.en_passent = Some(0x1 << (2 * 8 + from_col));
        }
        //Black en_passent valid
        else if from_pt == PieceType::BP && from_row == 6 && to_row == 4 {
            self.en_passent = Some(0x1 << (5 * 8 + from_col));
        } else {
            self.en_passent = None;
        }

        // Check for white castling
        if from_pt == PieceType::WK {
            //Kingside
            if from_ind == 4 && to_ind == 6 {
                self.is_w_castle = false;
                self.is_w_q_castle = false;
                self.w_r_bb &= !0x80;
                self.w_r_bb |= 0x20;
            }
            //Queenside
            else if from_ind == 4 && to_ind == 2 {
                self.is_w_castle = false;
                self.is_w_q_castle = false;
                self.w_r_bb &= !0x1;
                self.w_r_bb |= 0x8;
            }
        }
        // Check for black castling
        else if from_pt == PieceType::BK {
            //Kingside
            if from_ind == 60 && to_ind == 62 {
                self.is_b_castle = false;
                self.is_b_q_castle = false;
                self.b_r_bb &= !0x8000000000000000;
                self.b_r_bb |= 0x2000000000000000;
            }
            //Queenside
            if from_ind == 60 && to_ind == 58 {
                self.is_b_castle = false;
                self.is_b_q_castle = false;
                self.b_r_bb &= !0x100000000000000;
                self.b_r_bb |= 0x800000000000000;
            }
        }

        // Delete the landing square
        match to_pt {
            Some(ref pt) => match pt {
                PieceType::WP => {
                    self.w_p_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::WN => {
                    self.w_n_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::WB => {
                    self.w_b_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::WR => {
                    self.w_r_bb &= !(to_pos);
                    if to_pos == 0x1 {
                        self.is_w_q_castle = false;
                    } else if to_pos == 0x80 {
                        self.is_w_castle = false;
                    }
                    self.halfmove_clock = 0;
                }
                PieceType::WQ => {
                    self.w_q_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::WK => {
                    self.w_k_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::BP => {
                    self.b_p_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::BN => {
                    self.b_n_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::BB => {
                    self.b_b_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::BR => {
                    self.b_r_bb &= !(to_pos);
                    if to_pos == 0x100000000000000 {
                        self.is_b_q_castle = false;
                    } else if to_pos == 0x8000000000000000 {
                        self.is_b_castle = false;
                    }
                    self.halfmove_clock = 0;
                }
                PieceType::BQ => {
                    self.b_q_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
                PieceType::BK => {
                    self.b_k_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                }
            },
            None => (),
        }

        // Check for en-passent capture
        if to_pt == None
            && (from_pt == PieceType::WP || from_pt == PieceType::BP)
            && to_col != from_col
        {
            // Remove captured pawn
            if from_pt == PieceType::WP {
                self.b_p_bb &= !(0x1 << (to_ind - 8));
            } else {
                self.w_p_bb &= !(0x1 << (to_ind + 8));
            }
            self.halfmove_clock = 0;
        }

        // Move the piece
        match iter.next() {
            Some(c) => {
                // A promotion!
                // Add new piece
                match c {
                    'n' => self.b_n_bb |= to_pos,
                    'b' => self.b_b_bb |= to_pos,
                    'r' => self.b_r_bb |= to_pos,
                    'q' => self.b_q_bb |= to_pos,
                    'N' => self.w_n_bb |= to_pos,
                    'B' => self.w_b_bb |= to_pos,
                    'R' => self.w_r_bb |= to_pos,
                    'Q' => self.w_q_bb |= to_pos,
                    _ => panic!("Invalid promotion piece: {}", c),
                }
            }
            //Regular move
            None => match from_pt {
                PieceType::WP => self.w_p_bb |= to_pos,
                PieceType::WN => self.w_n_bb |= to_pos,
                PieceType::WB => self.w_b_bb |= to_pos,
                PieceType::WR => {
                    self.w_r_bb |= to_pos;
                    if from_pos == 0x1 {
                        self.is_w_q_castle = false;
                    } else if from_pos == 0x80 {
                        self.is_w_castle = false;
                    }
                }
                PieceType::WQ => self.w_q_bb |= to_pos,
                PieceType::WK => {
                    self.w_k_bb |= to_pos;
                    self.is_w_castle = false;
                    self.is_w_q_castle = false;
                }
                PieceType::BP => self.b_p_bb |= to_pos,
                PieceType::BN => self.b_n_bb |= to_pos,
                PieceType::BB => self.b_b_bb |= to_pos,
                PieceType::BR => {
                    self.b_r_bb |= to_pos;
                    if from_pos == 0x100000000000000 {
                        self.is_b_q_castle = false;
                    } else if from_pos == 0x8000000000000000 {
                        self.is_b_castle = false;
                    }
                }
                PieceType::BQ => self.b_q_bb |= to_pos,
                PieceType::BK => {
                    self.b_k_bb |= to_pos;
                    self.is_b_castle = false;
                    self.is_b_q_castle = false;
                }
            },
        }

        // Check if pawn move to reset halfmove counter
        if from_pt == PieceType::WP || from_pt == PieceType::BP {
            self.halfmove_clock = 0;
        }

        // Clear moving piece
        match from_pt {
            PieceType::WP => self.w_p_bb &= !(0x1 << from_ind),
            PieceType::WN => self.w_n_bb &= !(0x1 << from_ind),
            PieceType::WB => self.w_b_bb &= !(0x1 << from_ind),
            PieceType::WR => self.w_r_bb &= !(0x1 << from_ind),
            PieceType::WQ => self.w_q_bb &= !(0x1 << from_ind),
            PieceType::WK => self.w_k_bb &= !(0x1 << from_ind),
            PieceType::BP => self.b_p_bb &= !(0x1 << from_ind),
            PieceType::BN => self.b_n_bb &= !(0x1 << from_ind),
            PieceType::BB => self.b_b_bb &= !(0x1 << from_ind),
            PieceType::BR => self.b_r_bb &= !(0x1 << from_ind),
            PieceType::BQ => self.b_q_bb &= !(0x1 << from_ind),
            PieceType::BK => self.b_k_bb &= !(0x1 << from_ind),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut index: i32 = 56;
        let mut empties: u32 = 0;

        let mut string = String::new();
        // Stringify the board
        while index >= 0 {
            let pt = if self.w_p_bb & (0x1 << index) > 0 {
                Some(PieceType::WP)
            } else if self.w_n_bb & (0x1 << index) > 0 {
                Some(PieceType::WN)
            } else if self.w_b_bb & (0x1 << index) > 0 {
                Some(PieceType::WB)
            } else if self.w_r_bb & (0x1 << index) > 0 {
                Some(PieceType::WR)
            } else if self.w_q_bb & (0x1 << index) > 0 {
                Some(PieceType::WQ)
            } else if self.w_k_bb & (0x1 << index) > 0 {
                Some(PieceType::WK)
            } else if self.b_p_bb & (0x1 << index) > 0 {
                Some(PieceType::BP)
            } else if self.b_n_bb & (0x1 << index) > 0 {
                Some(PieceType::BN)
            } else if self.b_b_bb & (0x1 << index) > 0 {
                Some(PieceType::BB)
            } else if self.b_r_bb & (0x1 << index) > 0 {
                Some(PieceType::BR)
            } else if self.b_q_bb & (0x1 << index) > 0 {
                Some(PieceType::BQ)
            } else if self.b_k_bb & (0x1 << index) > 0 {
                Some(PieceType::BK)
            } else {
                None
            };

            if pt.is_some() && empties > 0 {
                string.push_str(&empties.to_string());
                empties = 0;
            }

            match pt {
                Some(x) => match x {
                    PieceType::WP => string.push('P'),
                    PieceType::WN => string.push('N'),
                    PieceType::WB => string.push('B'),
                    PieceType::WR => string.push('R'),
                    PieceType::WQ => string.push('Q'),
                    PieceType::WK => string.push('K'),
                    PieceType::BP => string.push('p'),
                    PieceType::BN => string.push('n'),
                    PieceType::BB => string.push('b'),
                    PieceType::BR => string.push('r'),
                    PieceType::BQ => string.push('q'),
                    PieceType::BK => string.push('k'),
                },
                None => empties += 1,
            }

            index += 1;
            if index % 8 == 0 {
                if empties > 0 {
                    string.push_str(&empties.to_string());
                }
                empties = 0;
                index -= 16;
                if index >= 0 {
                    string.push('/');
                }
            }
        }

        // Add other fen fields
        string.push(' ');
        match self.is_w_move {
            true => string.push('w'),
            false => string.push('b'),
        }

        string.push(' ');
        match self.is_w_castle {
            true => string.push('K'),
            false => (),
        }
        match self.is_w_q_castle {
            true => string.push('Q'),
            false => (),
        }
        match self.is_b_castle {
            true => string.push('k'),
            false => (),
        }
        match self.is_b_q_castle {
            true => string.push('q'),
            false => (),
        }
        if !(self.is_w_castle || self.is_w_q_castle || self.is_b_castle || self.is_b_q_castle) {
            string.push('-');
        }

        string.push(' ');
        match self.en_passent {
            Some(ep) => {
                let index = ep.trailing_zeros();
                let row = ((index / 8) + 1).to_string();
                let col = match index % 8 {
                    0 => 'a',
                    1 => 'b',
                    2 => 'c',
                    3 => 'd',
                    4 => 'e',
                    5 => 'f',
                    6 => 'g',
                    7 => 'h',
                    _ => panic!("Impossible!"),
                };
                string.push(col);
                string.push_str(&row);
            }
            None => string.push('-'),
        }

        string.push(' ');
        string.push_str(&self.halfmove_clock.to_string());

        string.push(' ');
        string.push_str(&self.fullmove_clock.to_string());

        write!(f, "{}", string)
    }
}
