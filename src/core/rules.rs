use std::fmt;

use super::types::{Board, Die, DieFace, DieKind, Field, Move, Outcome, Placement, Row, SLOTS};

pub fn score_row(row: &[Option<Die>; SLOTS]) -> u32 {
    let mut counts = [0u32; 7];
    for die in row.iter().flatten() {
        counts[die.face().value() as usize] += 1;
    }
    counts
        .iter()
        .enumerate()
        .filter(|(_, &n)| n > 0)
        .map(|(face, &n)| face as u32 * (2 * n - 1))
        .sum()
}

pub fn score_field(field: &Field) -> u32 {
    field.iter().map(score_row).sum()
}

pub fn evaluate_winner(player: &Field, opponent: &Field) -> Outcome {
    let mut player_rows = 0;
    let mut opponent_rows = 0;
    for row in Row::ALL {
        let ps = score_row(&player[row.index()]);
        let os = score_row(&opponent[row.index()]);
        if ps > os {
            player_rows += 1;
        } else if os > ps {
            opponent_rows += 1;
        }
    }

    if player_rows != opponent_rows {
        return if player_rows > opponent_rows {
            Outcome::Player
        } else {
            Outcome::Opponent
        };
    }

    match score_field(player).cmp(&score_field(opponent)) {
        std::cmp::Ordering::Greater => Outcome::Player,
        std::cmp::Ordering::Less => Outcome::Opponent,
        std::cmp::Ordering::Equal => Outcome::Draw,
    }
}

fn row_len(row: &[Option<Die>; SLOTS]) -> usize {
    row.iter().filter(|s| s.is_some()).count()
}

fn knockout_targets(their_row: &[Option<Die>; SLOTS], face: DieFace) -> Vec<usize> {
    their_row
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| match slot {
            Some(die) if !die.shield() && die.face() == face => Some(i),
            _ => None,
        })
        .collect()
}

pub fn legal_moves(
    mine: &Field,
    theirs: &Field,
    face: DieFace,
    kind: DieKind,
    first_die: bool,
) -> Vec<Move> {
    let mut moves = Vec::new();

    for row in Row::ALL {
        if row_len(&mine[row.index()]) < SLOTS {
            let knockout = if kind == DieKind::Normal {
                knockout_targets(&theirs[row.index()], face)
            } else {
                Vec::new()
            };
            moves.push(Move::new(Placement::mine(row), knockout));
        }
    }

    if kind == DieKind::Shielded && !first_die {
        for row in Row::ALL {
            if row_len(&theirs[row.index()]) < SLOTS {
                moves.push(Move::new(Placement::theirs(row), Vec::new()));
            }
        }
    }

    moves
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyMoveError {
    InvalidKnockoutSlot { row: Row, slot: usize },
    NoEmptySlot { placement: Placement },
}

impl fmt::Display for ApplyMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKnockoutSlot { row, slot } => {
                write!(f, "invalid knockout slot {slot} in row {}", row.index())
            }
            Self::NoEmptySlot { placement } => {
                write!(
                    f,
                    "no empty slot on {:?} row {}",
                    placement.board(),
                    placement.row().index()
                )
            }
        }
    }
}

impl std::error::Error for ApplyMoveError {}

pub fn apply_move(
    mine: &Field,
    theirs: &Field,
    die: Die,
    mv: &Move,
) -> Result<(Field, Field), ApplyMoveError> {
    let mut new_mine = *mine;
    let mut new_theirs = *theirs;
    let row_idx = mv.placement().row().index();

    for &idx in mv.knockout() {
        if idx >= SLOTS {
            return Err(ApplyMoveError::InvalidKnockoutSlot {
                row: mv.placement().row(),
                slot: idx,
            });
        }
        new_theirs[row_idx][idx] = None;
    }

    let target = match mv.placement().board() {
        Board::Mine => &mut new_mine,
        Board::Theirs => &mut new_theirs,
    };
    let row = &mut target[row_idx];
    let Some(slot) = row.iter().position(Option::is_none) else {
        return Err(ApplyMoveError::NoEmptySlot {
            placement: mv.placement(),
        });
    };
    row[slot] = Some(die);

    Ok((new_mine, new_theirs))
}

pub(crate) fn field_full(field: &Field) -> bool {
    field.iter().all(|row| row.iter().all(Option::is_some))
}

pub(crate) fn heuristic_eval(mine: &Field, theirs: &Field) -> i32 {
    let mut rows_margin: i32 = 0;
    for row in Row::ALL {
        let ms = score_row(&mine[row.index()]);
        let ts = score_row(&theirs[row.index()]);
        if ms > ts {
            rows_margin += 1;
        } else if ts > ms {
            rows_margin -= 1;
        }
    }
    let total_margin = score_field(mine) as i32 - score_field(theirs) as i32;
    rows_margin * 1000 + total_margin
}
