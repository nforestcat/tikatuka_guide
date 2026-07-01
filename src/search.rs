use crate::core::{
    apply_move, field_full, heuristic_eval, legal_moves, Die, DieFace, DieKind, Field, Move,
};

mod advice;

pub use advice::{
    recommend_after_reroll, recommend_move, recommend_move_search, recommend_turn, RerollChoice,
    TurnAdvice, TurnInput,
};

pub const DEFAULT_DEPTH: u32 = 3;

#[derive(Clone, Copy)]
pub(crate) struct Search {
    pub(crate) human: Field,
    pub(crate) cpu: Field,
    pub(crate) human_to_move: bool,
    pub(crate) human_reroll: bool,
    pub(crate) cpu_reroll: bool,
}

impl Search {
    pub(crate) fn from_input(input: TurnInput) -> Self {
        Self {
            human: input.mine,
            cpu: input.theirs,
            human_to_move: true,
            human_reroll: input.human_reroll,
            cpu_reroll: input.cpu_reroll,
        }
    }

    fn mover_full(&self) -> bool {
        field_full(if self.human_to_move {
            &self.human
        } else {
            &self.cpu
        })
    }

    fn mover_reroll(&self) -> bool {
        if self.human_to_move {
            self.human_reroll
        } else {
            self.cpu_reroll
        }
    }

    pub(crate) fn spend_reroll(mut self) -> Self {
        if self.human_to_move {
            self.human_reroll = false;
        } else {
            self.cpu_reroll = false;
        }
        self
    }

    fn mover_boards(&self) -> (Field, Field) {
        if self.human_to_move {
            (self.human, self.cpu)
        } else {
            (self.cpu, self.human)
        }
    }

    fn with_mover_boards(mut self, mine: Field, theirs: Field) -> Self {
        if self.human_to_move {
            self.human = mine;
            self.cpu = theirs;
        } else {
            self.cpu = mine;
            self.human = theirs;
        }
        self
    }

    pub(crate) fn flip(mut self) -> Self {
        self.human_to_move = !self.human_to_move;
        self
    }
}

fn pick(human_to_move: bool, a: f64, b: f64) -> f64 {
    if human_to_move {
        a.max(b)
    } else {
        a.min(b)
    }
}

pub(crate) fn place_value(st: Search, face: DieFace, depth: u32) -> f64 {
    let (mine, theirs) = st.mover_boards();
    let die = Die::new(face, false);
    let moves = legal_moves(&mine, &theirs, face, DieKind::Normal, false);
    let values = moves.iter().filter_map(|mv| {
        let (nmine, ntheirs) = apply_move(&mine, &theirs, die, mv).ok()?;
        let ns = st.with_mover_boards(nmine, ntheirs);
        Some(if mv.knockout().is_empty() {
            turn_value(ns.flip(), depth - 1)
        } else {
            bonus_value(ns, depth - 1)
        })
    });

    if st.human_to_move {
        values.fold(f64::NEG_INFINITY, f64::max)
    } else {
        values.fold(f64::INFINITY, f64::min)
    }
}

pub(crate) fn bonus_value(st: Search, next_depth: u32) -> f64 {
    let (mine, theirs) = st.mover_boards();
    let mut sum = 0.0;

    for face in DieFace::ALL {
        let die = Die::new(face, true);
        let moves = legal_moves(&mine, &theirs, face, DieKind::Shielded, false);
        let values = moves.iter().filter_map(|mv| {
            let (nmine, ntheirs) = apply_move(&mine, &theirs, die, mv).ok()?;
            Some(turn_value(
                st.with_mover_boards(nmine, ntheirs).flip(),
                next_depth,
            ))
        });
        let value = if moves.is_empty() {
            turn_value(st.flip(), next_depth)
        } else if st.human_to_move {
            values.fold(f64::NEG_INFINITY, f64::max)
        } else {
            values.fold(f64::INFINITY, f64::min)
        };
        sum += value;
    }

    sum / 6.0
}

fn decide_roll(st: Search, face: DieFace, depth: u32) -> f64 {
    let keep = place_value(st, face, depth);
    if !st.mover_reroll() {
        return keep;
    }

    let spent = st.spend_reroll();
    let keep_after = place_value(spent, face, depth);
    let mut sum = 0.0;
    for new_face in DieFace::ALL {
        sum += pick(
            st.human_to_move,
            keep_after,
            place_value(spent, new_face, depth),
        );
    }
    pick(st.human_to_move, keep, sum / 6.0)
}

pub(crate) fn turn_value(st: Search, depth: u32) -> f64 {
    if depth == 0 || (field_full(&st.human) && field_full(&st.cpu)) {
        return heuristic_eval(&st.human, &st.cpu) as f64;
    }
    if st.mover_full() {
        return turn_value(st.flip(), depth - 1);
    }

    let mut sum = 0.0;
    for face in DieFace::ALL {
        sum += decide_roll(st, face, depth);
    }
    sum / 6.0
}

pub(crate) fn best_placement(
    st: Search,
    face: DieFace,
    kind: DieKind,
    first_die: bool,
    depth: u32,
) -> Option<(f64, Move)> {
    let (mine, theirs) = st.mover_boards();
    let die = Die::new(face, matches!(kind, DieKind::Shielded));
    legal_moves(&mine, &theirs, face, kind, first_die)
        .into_iter()
        .filter_map(|mv| {
            let (nmine, ntheirs) = apply_move(&mine, &theirs, die, &mv).ok()?;
            let ns = st.with_mover_boards(nmine, ntheirs);
            let value = if mv.knockout().is_empty() {
                turn_value(ns.flip(), depth - 1)
            } else {
                bonus_value(ns, depth - 1)
            };
            Some((value, mv))
        })
        .fold(None, |best: Option<(f64, Move)>, (value, mv)| match best {
            Some((best_value, _)) if best_value >= value => best,
            _ => Some((value, mv)),
        })
}
