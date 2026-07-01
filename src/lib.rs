//! Tikatuka core rule engine: pure game logic, no screen/UI dependencies.

mod core;
#[cfg(test)]
mod core_tests;
mod search;
#[cfg(test)]
mod search_tests;

pub use crate::core::{
    apply_move, evaluate_winner, legal_moves, score_field, score_row, ApplyMoveError, Board, Die,
    DieFace, DieFaceError, DieKind, Field, Move, Outcome, Placement, Row,
};
pub use crate::search::{
    recommend_after_reroll, recommend_move, recommend_move_search, recommend_turn, RerollChoice,
    TurnAdvice, TurnInput, DEFAULT_DEPTH,
};
