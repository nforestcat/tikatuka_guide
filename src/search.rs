use std::collections::HashMap;

use crate::core::{
    apply_move, field_full, heuristic_eval, legal_moves, CurrentDie, DieFace, Field, Move,
};

mod advice;

pub use advice::{
    recommend_after_reroll, recommend_move, recommend_move_search, recommend_turn, RerollChoice,
    TurnAdvice, TurnInput,
};

pub const DEFAULT_DEPTH: u32 = 3;

#[derive(Default)]
struct EvalCache {
    turns: HashMap<(Search, u32), f64>,
    placements: HashMap<(Search, CurrentDie, u32), f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

fn choose_value(human_to_move: bool, best: f64, value: f64) -> f64 {
    pick(human_to_move, best, value)
}

pub(crate) fn place_value(st: Search, current: CurrentDie, depth: u32) -> f64 {
    let mut cache = EvalCache::default();
    place_value_cached(st, current, depth, &mut cache)
}

fn place_value_cached(st: Search, current: CurrentDie, depth: u32, cache: &mut EvalCache) -> f64 {
    if let Some(&value) = cache.placements.get(&(st, current, depth)) {
        return value;
    }

    let (mine, theirs) = st.mover_boards();
    let die = current.placed_die();
    let mut best = if st.human_to_move {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };

    for mv in legal_moves(&mine, &theirs, current) {
        let Ok((nmine, ntheirs)) = apply_move(&mine, &theirs, die, &mv) else {
            continue;
        };
        let ns = st.with_mover_boards(nmine, ntheirs);
        let value = if mv.knockout().is_empty() {
            turn_value_cached(ns.flip(), depth - 1, cache)
        } else {
            bonus_value_cached(ns, depth - 1, cache)
        };
        best = choose_value(st.human_to_move, best, value);
    }

    cache.placements.insert((st, current, depth), best);
    best
}

#[cfg(test)]
pub(crate) fn bonus_value(st: Search, next_depth: u32) -> f64 {
    let mut cache = EvalCache::default();
    bonus_value_cached(st, next_depth, &mut cache)
}

fn bonus_value_cached(st: Search, next_depth: u32, cache: &mut EvalCache) -> f64 {
    let (mine, theirs) = st.mover_boards();
    let mut sum = 0.0;

    for face in DieFace::ALL {
        let current = CurrentDie::shielded(face);
        let die = current.placed_die();
        let moves = legal_moves(&mine, &theirs, current);
        let mut value = if st.human_to_move {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        if moves.is_empty() {
            value = turn_value_cached(st.flip(), next_depth, cache);
        }
        for mv in moves {
            let Ok((nmine, ntheirs)) = apply_move(&mine, &theirs, die, &mv) else {
                continue;
            };
            let next = turn_value_cached(
                st.with_mover_boards(nmine, ntheirs).flip(),
                next_depth,
                cache,
            );
            value = choose_value(st.human_to_move, value, next);
        }
        sum += value;
    }

    sum / 6.0
}

fn decide_roll(st: Search, current: CurrentDie, depth: u32, cache: &mut EvalCache) -> f64 {
    let keep = place_value_cached(st, current, depth, cache);
    if !st.mover_reroll() {
        return keep;
    }

    let spent = st.spend_reroll();
    let keep_after = place_value_cached(spent, current, depth, cache);
    let mut sum = 0.0;
    for new_face in DieFace::ALL {
        sum += pick(
            st.human_to_move,
            keep_after,
            place_value_cached(spent, current.with_face(new_face), depth, cache),
        );
    }
    pick(st.human_to_move, keep, sum / 6.0)
}

#[cfg(test)]
pub(crate) fn turn_value(st: Search, depth: u32) -> f64 {
    let mut cache = EvalCache::default();
    turn_value_cached(st, depth, &mut cache)
}

fn turn_value_cached(st: Search, depth: u32, cache: &mut EvalCache) -> f64 {
    if let Some(&value) = cache.turns.get(&(st, depth)) {
        return value;
    }

    let value = if depth == 0 || (field_full(&st.human) && field_full(&st.cpu)) {
        heuristic_eval(&st.human, &st.cpu) as f64
    } else if st.mover_full() {
        turn_value_cached(st.flip(), depth, cache)
    } else {
        let mut sum = 0.0;
        for face in DieFace::ALL {
            sum += decide_roll(st, CurrentDie::normal(face), depth, cache);
        }
        sum / 6.0
    };
    cache.turns.insert((st, depth), value);
    value
}

pub(crate) fn best_placement(st: Search, current: CurrentDie, depth: u32) -> Option<(f64, Move)> {
    let mut cache = EvalCache::default();
    best_placement_cached(st, current, depth, &mut cache)
}

fn best_placement_cached(
    st: Search,
    current: CurrentDie,
    depth: u32,
    cache: &mut EvalCache,
) -> Option<(f64, Move)> {
    let (mine, theirs) = st.mover_boards();
    let die = current.placed_die();
    legal_moves(&mine, &theirs, current)
        .into_iter()
        .filter_map(|mv| {
            let (nmine, ntheirs) = apply_move(&mine, &theirs, die, &mv).ok()?;
            let ns = st.with_mover_boards(nmine, ntheirs);
            let value = if mv.knockout().is_empty() {
                turn_value_cached(ns.flip(), depth - 1, cache)
            } else {
                bonus_value_cached(ns, depth - 1, cache)
            };
            Some((value, mv))
        })
        .fold(None, |best: Option<(f64, Move)>, (value, mv)| match best {
            Some((best_value, _)) if best_value >= value => best,
            _ => Some((value, mv)),
        })
}
