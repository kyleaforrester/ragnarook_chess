
pub struct Board {
    board: String,
}

impl Board {
    pub fn new(fen: &str) -> Board {
        Board {
            board: String::from("Yo"),
        }
    }
}
