use std::fmt;

pub const ROWS: usize = 3;
pub const SLOTS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DieFace(u8);

impl DieFace {
    pub const ALL: [Self; 6] = [Self(1), Self(2), Self(3), Self(4), Self(5), Self(6)];

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DieFaceError {
    value: u8,
}

impl DieFaceError {
    pub const fn value(self) -> u8 {
        self.value
    }
}

impl fmt::Display for DieFaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "die face must be 1..=6, got {}", self.value)
    }
}

impl std::error::Error for DieFaceError {}

impl TryFrom<u8> for DieFace {
    type Error = DieFaceError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=6).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DieFaceError { value })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Die {
    face: DieFace,
    shield: bool,
}

impl Die {
    pub const fn new(face: DieFace, shield: bool) -> Self {
        Self { face, shield }
    }

    pub const fn face(self) -> DieFace {
        self.face
    }

    pub const fn shield(self) -> bool {
        self.shield
    }
}

pub type Field = [[Option<Die>; SLOTS]; ROWS];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Row(usize);

impl Row {
    pub const ALL: [Self; ROWS] = [Self(0), Self(1), Self(2)];

    pub const fn index(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for Row {
    type Error = usize;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < ROWS {
            Ok(Self(value))
        } else {
            Err(value)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Player,
    Opponent,
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DieKind {
    Normal,
    Shielded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Board {
    Mine,
    Theirs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    board: Board,
    row: Row,
}

impl Placement {
    pub const fn new(board: Board, row: Row) -> Self {
        Self { board, row }
    }

    pub const fn mine(row: Row) -> Self {
        Self::new(Board::Mine, row)
    }

    pub const fn theirs(row: Row) -> Self {
        Self::new(Board::Theirs, row)
    }

    pub const fn board(self) -> Board {
        self.board
    }

    pub const fn row(self) -> Row {
        self.row
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    placement: Placement,
    knockout: Vec<usize>,
}

impl Move {
    pub(crate) fn new(placement: Placement, knockout: Vec<usize>) -> Self {
        Self {
            placement,
            knockout,
        }
    }

    pub const fn placement(&self) -> Placement {
        self.placement
    }

    pub fn knockout(&self) -> &[usize] {
        &self.knockout
    }
}
