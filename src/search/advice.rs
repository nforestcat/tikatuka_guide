use crate::core::{apply_move, heuristic_eval, legal_moves, Die, DieFace, DieKind, Field, Move};

use super::{best_placement, place_value, Search};

pub fn recommend_move(
    mine: &Field,
    theirs: &Field,
    face: DieFace,
    kind: DieKind,
    first_die: bool,
) -> Option<Move> {
    let die = Die::new(face, matches!(kind, DieKind::Shielded));
    legal_moves(mine, theirs, face, kind, first_die)
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
    pub face: DieFace,
    pub kind: DieKind,
    pub first_die: bool,
    pub human_reroll: bool,
    pub cpu_reroll: bool,
    pub depth: u32,
}

impl TurnInput {
    pub(crate) fn search_depth(self) -> u32 {
        self.depth.max(1)
    }

    fn with_face(mut self, face: DieFace) -> Self {
        self.face = face;
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
        input.face,
        input.kind,
        input.first_die,
        input.search_depth(),
    )
    .map(|(_, mv)| mv)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RerollChoice {
    pub face: DieFace,
    pub placement: Move,
}

pub fn recommend_after_reroll(input: TurnInput, new_face: DieFace) -> Option<RerollChoice> {
    let spent = input.with_spent_human_reroll();
    let old = best_placement(
        Search::from_input(spent),
        spent.face,
        spent.kind,
        spent.first_die,
        spent.search_depth(),
    );
    let new = best_placement(
        Search::from_input(spent.with_face(new_face)),
        new_face,
        spent.kind,
        spent.first_die,
        spent.search_depth(),
    );

    match (old, new) {
        (Some((old_value, old_move)), Some((new_value, _))) if old_value >= new_value => {
            Some(RerollChoice {
                face: spent.face,
                placement: old_move,
            })
        }
        (Some(_), Some((_, new_move))) | (None, Some((_, new_move))) => Some(RerollChoice {
            face: new_face,
            placement: new_move,
        }),
        (Some((_, old_move)), None) => Some(RerollChoice {
            face: spent.face,
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
    let (keep_val, best_move) = best_placement(st, input.face, input.kind, input.first_die, depth)?;

    if input.human_reroll && input.kind == DieKind::Normal {
        let spent = st.spend_reroll();
        let keep_after = place_value(spent, input.face, depth);
        let mut sum = 0.0;
        for new_face in DieFace::ALL {
            sum += keep_after.max(place_value(spent, new_face, depth));
        }
        if sum / 6.0 > keep_val {
            return Some(TurnAdvice::Reroll);
        }
    }

    Some(TurnAdvice::Place(best_move))
}
