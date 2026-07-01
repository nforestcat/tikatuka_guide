use crate::core::{apply_move, heuristic_eval, legal_moves, CurrentDie, Field, Move};

use super::{best_placement, place_value, Search};

pub fn recommend_move(mine: &Field, theirs: &Field, current: CurrentDie) -> Option<Move> {
    let die = current.placed_die();
    legal_moves(mine, theirs, current)
        .into_iter()
        .filter_map(|mv| {
            let (new_mine, new_theirs) = apply_move(mine, theirs, die, &mv).ok()?;
            Some((heuristic_eval(&new_mine, &new_theirs), mv))
        })
        .fold(None, |best: Option<(i32, Move)>, (rank, mv)| match best {
            Some((best_rank, _)) if best_rank >= rank => best,
            _ => Some((rank, mv)),
        })
        .map(|(_, mv)| mv)
}

#[derive(Clone, Copy, Debug)]
pub struct TurnInput {
    pub mine: Field,
    pub theirs: Field,
    pub current: CurrentDie,
    pub human_reroll: bool,
    pub cpu_reroll: bool,
    pub depth: u32,
}

impl TurnInput {
    pub(crate) fn search_depth(self) -> u32 {
        self.depth.max(1)
    }

    fn with_current(mut self, current: CurrentDie) -> Self {
        self.current = current;
        self
    }

    fn with_spent_human_reroll(mut self) -> Self {
        self.human_reroll = false;
        self
    }
}

pub fn recommend_move_search(input: TurnInput) -> Option<Move> {
    best_placement(
        Search::from_input(input),
        input.current,
        input.search_depth(),
    )
    .map(|(_, mv)| mv)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RerollChoice {
    pub current: CurrentDie,
    pub placement: Move,
}

pub fn recommend_after_reroll(input: TurnInput, new_current: CurrentDie) -> Option<RerollChoice> {
    let spent = input.with_spent_human_reroll();
    let old = best_placement(
        Search::from_input(spent),
        spent.current,
        spent.search_depth(),
    );
    let new = best_placement(
        Search::from_input(spent.with_current(new_current)),
        new_current,
        spent.search_depth(),
    );

    match (old, new) {
        (Some((old_value, old_move)), Some((new_value, _))) if old_value >= new_value => {
            Some(RerollChoice {
                current: spent.current,
                placement: old_move,
            })
        }
        (Some(_), Some((_, new_move))) | (None, Some((_, new_move))) => Some(RerollChoice {
            current: new_current,
            placement: new_move,
        }),
        (Some((_, old_move)), None) => Some(RerollChoice {
            current: spent.current,
            placement: old_move,
        }),
        (None, None) => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnAdvice {
    Place(Move),
    Reroll,
}

pub fn recommend_turn(input: TurnInput) -> Option<TurnAdvice> {
    let depth = input.search_depth();
    let st = Search::from_input(input);
    let (keep_val, best_move) = best_placement(st, input.current, depth)?;

    if input.human_reroll {
        let spent = st.spend_reroll();
        let keep_after = place_value(spent, input.current, depth);
        let mut sum = 0.0;
        for new_face in crate::core::DieFace::ALL {
            sum += keep_after.max(place_value(spent, input.current.with_face(new_face), depth));
        }
        if sum / 6.0 > keep_val {
            return Some(TurnAdvice::Reroll);
        }
    }

    Some(TurnAdvice::Place(best_move))
}
