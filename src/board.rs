
#[derive(Eq,PartialEq)]
pub struct Board {
    w_p_bb: u64,
    w_n_bb: u64,
    w_b_bb: u64,
    w_r_bb: u64,
    w_q_bb: u64,
    w_k_bb: u64,
    b_p_bb: u64,
    b_n_bb: u64,
    b_b_bb: u64,
    b_r_bb: u64,
    b_q_bb: u64,
    b_k_bb: u64,
    pub is_w_move: bool,
    is_w_castle: bool,
    is_w_q_castle: bool,
    is_b_castle: bool,
    is_b_q_castle: bool,
    en_passent: u64,
    halfmove_clock: u32,
    fullmove_clock: u32,
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
        let fen_tokens: Vec<String> = fen.split_ascii_whitespace().map(|x| String::from(x)).collect();

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
                n @ '1'..='8' => col += n.to_digit(10).unwrap(),
                '/' => {
                    row -= 1;
                    col = 0;
                },
                _ => panic!("Invalid character in fen board: {}", c),
            }
        }

        let is_white_move = if fen_tokens[1] == "w" {
                true
            }
            else {
                false
            };

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
                _ => panic!("Invalid character in fen castling rights: {}", c),
            }
        }

        let mut en_passent = 0;
        let en_p_str = &fen_tokens[3];
        if en_p_str.chars().count() == 2 {
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

            en_passent = 0x1 << (row * 8 + col);
        }

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

    pub fn do_move(&self, mov: &str) {
        let mut iter = mov.chars();
        let from_col = iter.next().unwrap();
        let from_row = iter.next().unwrap().to_digit(10).unwrap();
        let to_col = iter.next().unwrap();
        let to_row = iter.next().unwrap().to_digit(10).unwrap();

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
            PieceType::WP | PieceType::WN | PieceType::WB | PieceType::WR | PieceType::WQ | PieceType::WK => self.is_w_move = false,
            _ => self.is_w_move = true,
        }

        //Increment move counters
        self.halfmove_clock += 1;
        self.fullmove_clock += 1;

        // White en_passent valid
        if from_pt == PieceType::WP && from_row == 1 && to_row == 3 {
            self.en_passent = 0x1 << (2 * 8 + from_col);
        }
        //Black en_passent valid
        else if from_pt == PieceType::BP && from_row == 6 && to_row == 4 {
            self.en_passent = 0x1 << (5 * 8 + from_col);
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
            Some(pt) => match pt {
                WP => {
                    self.w_p_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                WN => {
                    self.w_n_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                WB => {
                    self.w_b_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                WR => {
                    self.w_r_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                WQ => {
                    self.w_q_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                WK => {
                    self.w_k_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BP => {
                    self.b_p_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BN => {
                    self.b_n_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BB => {
                    self.b_b_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BR => {
                    self.b_r_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BQ => {
                    self.b_q_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
                BK => {
                    self.b_k_bb &= !(to_pos);
                    self.halfmove_clock = 0;
                },
            },
            None => (),
        }

        // Check for en-passent capture
        if to_pt == None && (from_pt == PieceType::WP || from_pt == PieceType::BP) && to_col != from_col {
            // Remove captured pawn
            if from_pt == PieceType::WP {
                self.b_p_bb &= !(0x1 << (to_ind - 8));
            }
            else {
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
                    'n' => self.b_n_bb |= 0x1 << to_ind,
                    'b' => self.b_b_bb |= 0x1 << to_ind,
                    'r' => self.b_r_bb |= 0x1 << to_ind,
                    'q' => self.b_q_bb |= 0x1 << to_ind,
                    'N' => self.w_n_bb |= 0x1 << to_ind,
                    'B' => self.w_b_bb |= 0x1 << to_ind,
                    'R' => self.w_r_bb |= 0x1 << to_ind,
                    'Q' => self.w_q_bb |= 0x1 << to_ind,
                    _ => panic!("Invalid promotion piece: {}", c),
                }
            },
            //Regular move
            None => match from_pt {
                WP => self.w_p_bb |= 0x1 << to_ind,
                WN => self.w_n_bb |= 0x1 << to_ind,
                WB => self.w_b_bb |= 0x1 << to_ind,
                WR => self.w_r_bb |= 0x1 << to_ind,
                WQ => self.w_q_bb |= 0x1 << to_ind,
                WK => self.w_k_bb |= 0x1 << to_ind,
                BP => self.b_p_bb |= 0x1 << to_ind,
                BN => self.b_n_bb |= 0x1 << to_ind,
                BB => self.b_b_bb |= 0x1 << to_ind,
                BR => self.b_r_bb |= 0x1 << to_ind,
                BQ => self.b_q_bb |= 0x1 << to_ind,
                BK => self.b_k_bb |= 0x1 << to_ind,
            },
        }

        // Check if pawn move to reset halfmove counter
        if from_pt == PieceType::WP || from_pt == PieceType::BP {
            self.halfmove_clock = 0;
        }

        // Clear moving piece
        match from_pt {
            WP => self.w_p_bb &= !(0x1 << from_ind),
            WN => self.w_n_bb &= !(0x1 << from_ind),
            WB => self.w_b_bb &= !(0x1 << from_ind),
            WR => self.w_r_bb &= !(0x1 << from_ind),
            WQ => self.w_q_bb &= !(0x1 << from_ind),
            WK => self.w_k_bb &= !(0x1 << from_ind),
            BP => self.b_p_bb &= !(0x1 << from_ind),
            BN => self.b_n_bb &= !(0x1 << from_ind),
            BB => self.b_b_bb &= !(0x1 << from_ind),
            BR => self.b_r_bb &= !(0x1 << from_ind),
            BQ => self.b_q_bb &= !(0x1 << from_ind),
            BK => self.b_k_bb &= !(0x1 << from_ind),
        }
    }
}
