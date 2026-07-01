use crate::core::{heuristic_eval, score_field, score_row};
use crate::search::{bonus_value, turn_value, Search};
use crate::{
    apply_move, legal_moves, recommend_after_reroll, recommend_move, recommend_move_search,
    recommend_turn, Board, Die, DieFace, DieKind, Field, Move, Placement, Row, TurnAdvice,
    TurnInput, DEFAULT_DEPTH,
};

fn face(value: u8) -> DieFace {
    DieFace::try_from(value).unwrap()
}

fn die(value: u8, shield: bool) -> Die {
    Die::new(face(value), shield)
}

fn row(faces: &[u8]) -> [Option<Die>; 3] {
    let mut row = [None, None, None];
    for (i, &value) in faces.iter().enumerate() {
        row[i] = Some(die(value, false));
    }
    row
}

fn input(face_value: u8, mine: Field, theirs: Field, depth: u32) -> TurnInput {
    TurnInput {
        mine,
        theirs,
        face: face(face_value),
        kind: DieKind::Normal,
        first_die: false,
        human_reroll: true,
        cpu_reroll: true,
        depth,
    }
}

fn search_state(human: Field, cpu: Field, human_to_move: bool) -> Search {
    Search {
        human,
        cpu,
        human_to_move,
        human_reroll: true,
        cpu_reroll: true,
    }
}

#[test]
fn recommend_move_none_on_full_own_board_with_normal_die() {
    let mine: Field = [row(&[1, 1, 1]), row(&[2, 2, 2]), row(&[3, 3, 3])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    assert_eq!(
        recommend_move(&mine, &theirs, face(5), DieKind::Normal, false),
        None
    );
}

#[test]
fn recommend_move_prefers_winning_knockout() {
    let mine: Field = [row(&[]), row(&[]), row(&[])];
    let theirs: Field = [row(&[6, 6, 6]), row(&[]), row(&[])];
    let mv = recommend_move(&mine, &theirs, face(6), DieKind::Normal, false).unwrap();

    assert_eq!(mv.placement(), Placement::mine(Row::ALL[0]));
    assert_eq!(mv.knockout(), &[0, 1, 2]);
}

#[test]
fn turn_value_depth_zero_is_the_heuristic() {
    let human: Field = [row(&[5, 5]), row(&[3]), row(&[])];
    let cpu: Field = [row(&[2]), row(&[6, 6]), row(&[1])];
    let st = search_state(human, cpu, true);
    assert_eq!(turn_value(st, 0), heuristic_eval(&human, &cpu) as f64);
}

#[test]
fn search_recommends_obvious_knockout() {
    let mine: Field = [row(&[]), row(&[]), row(&[])];
    let theirs: Field = [row(&[6, 6, 6]), row(&[]), row(&[])];
    let mv = recommend_move_search(input(6, mine, theirs, 2)).unwrap();
    assert_eq!(mv.placement(), Placement::mine(Row::ALL[0]));
    assert_eq!(mv.knockout(), &[0, 1, 2]);
}

#[test]
fn reroll_recommended_for_a_bad_face_when_available() {
    let empty: Field = [row(&[]), row(&[]), row(&[])];
    assert_eq!(
        recommend_turn(input(1, empty, empty, 1)),
        Some(TurnAdvice::Reroll)
    );
}

#[test]
fn no_reroll_when_unavailable_yields_a_placement() {
    let empty: Field = [row(&[]), row(&[]), row(&[])];
    let mut turn = input(1, empty, empty, 1);
    turn.human_reroll = false;
    assert!(matches!(recommend_turn(turn), Some(TurnAdvice::Place(_))));
}

#[test]
fn recommend_after_reroll_selects_the_new_face_when_better() {
    let empty: Field = [row(&[]), row(&[]), row(&[])];
    let choice = recommend_after_reroll(input(1, empty, empty, 1), face(6)).unwrap();
    assert_eq!(choice.face, face(6));
}

#[test]
fn bonus_value_falls_back_when_both_boards_full() {
    let human: Field = [row(&[1, 1, 1]), row(&[2, 2, 2]), row(&[3, 3, 3])];
    let cpu: Field = [row(&[4, 4, 4]), row(&[5, 5, 5]), row(&[6, 6, 6])];
    let st = search_state(human, cpu, true);
    assert_eq!(bonus_value(st, 2), turn_value(st.flip(), 2));
}

#[test]
fn bonus_value_is_finite_when_placement_exists() {
    let human: Field = [row(&[]), row(&[3]), row(&[])];
    let cpu: Field = [row(&[]), row(&[1]), row(&[])];
    assert!(bonus_value(search_state(human, cpu, true), 2).is_finite());
    assert!(bonus_value(search_state(human, cpu, false), 2).is_finite());
}

#[test]
fn bonus_search_considers_opponent_board_when_it_is_best() {
    let human: Field = [row(&[6, 6, 6]), row(&[]), row(&[])];
    let cpu: Field = [row(&[1, 1]), row(&[]), row(&[])];
    let st = search_state(human, cpu, true);

    let full_search = bonus_value(st, 2);
    let own_only = {
        let mut sum = 0.0;
        for face in DieFace::ALL {
            let die = Die::new(face, true);
            let moves: Vec<Move> = legal_moves(&human, &cpu, face, DieKind::Shielded, false)
                .into_iter()
                .filter(|mv| mv.placement().board() == Board::Mine)
                .collect();
            let value = moves
                .iter()
                .filter_map(|mv| {
                    let (new_human, new_cpu) = apply_move(&human, &cpu, die, mv).ok()?;
                    Some(turn_value(search_state(new_human, new_cpu, false), 2))
                })
                .fold(f64::NEG_INFINITY, f64::max);
            sum += value;
        }
        sum / 6.0
    };

    assert!(full_search > own_only);
}

#[test]
fn search_returns_none_on_full_board() {
    let mine: Field = [row(&[1, 1, 1]), row(&[2, 2, 2]), row(&[3, 3, 3])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    assert_eq!(
        recommend_move_search(input(5, mine, theirs, DEFAULT_DEPTH)),
        None
    );
}

#[test]
fn search_returns_a_legal_move_midgame() {
    let mine: Field = [row(&[3]), row(&[2, 4]), row(&[])];
    let theirs: Field = [row(&[5, 5]), row(&[1]), row(&[6])];
    let turn = input(5, mine, theirs, 2);
    let mv = recommend_move_search(turn).unwrap();
    let legal = legal_moves(&turn.mine, &turn.theirs, turn.face, DieKind::Normal, false);
    assert!(legal.contains(&mv));
}

#[test]
fn heuristic_keeps_row_priority_above_total_score() {
    let mine: Field = [row(&[6, 6]), row(&[]), row(&[])];
    let theirs: Field = [row(&[1]), row(&[6, 6, 6]), row(&[])];
    assert!(heuristic_eval(&mine, &theirs) < 0);
    assert_eq!(score_row(&mine[0]), 18);
    assert_eq!(score_field(&theirs), 31);
}
