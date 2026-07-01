use crate::{
    apply_move, evaluate_winner, legal_moves, score_field, score_row, ApplyMoveError, Board, Die,
    DieFace, DieKind, Field, Outcome, Placement, Row,
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

fn srow(dice: &[(u8, bool)]) -> [Option<Die>; 3] {
    let mut row = [None, None, None];
    for (i, &(value, shield)) in dice.iter().enumerate() {
        row[i] = Some(die(value, shield));
    }
    row
}

#[test]
fn die_face_rejects_invalid_values() {
    assert!(DieFace::try_from(0).is_err());
    assert!(DieFace::try_from(7).is_err());
    assert_eq!(face(6).value(), 6);
}

#[test]
fn scoring_duplicate_bonus() {
    assert_eq!(score_row(&row(&[5])), 5);
    assert_eq!(score_row(&row(&[5, 5])), 15);
    assert_eq!(score_row(&row(&[5, 5, 5])), 25);
    assert_eq!(score_row(&row(&[4, 4])), 12);
    assert_eq!(score_row(&row(&[6, 5, 3])), 14);
    assert_eq!(score_row(&row(&[5, 5, 3])), 18);
    assert_eq!(score_row(&row(&[])), 0);
}

#[test]
fn shield_does_not_change_score() {
    let mut row = row(&[5, 5, 5]);
    row[0] = Some(die(5, true));
    assert_eq!(score_row(&row), 25);
}

#[test]
fn winner_by_row_count() {
    let player: Field = [row(&[6]), row(&[6]), row(&[1])];
    let opponent: Field = [row(&[1]), row(&[1]), row(&[6])];
    assert_eq!(evaluate_winner(&player, &opponent), Outcome::Player);
}

#[test]
fn worked_example_from_spec() {
    let player: Field = [row(&[5, 5, 5]), row(&[6, 5, 3]), row(&[4, 4])];
    let opponent: Field = [row(&[6, 4, 3]), row(&[6, 5, 3]), row(&[6, 6])];
    assert_eq!(score_field(&player), 51);
    assert_eq!(score_field(&opponent), 45);
    assert_eq!(evaluate_winner(&player, &opponent), Outcome::Player);
}

#[test]
fn full_draw() {
    let player: Field = [row(&[6, 5]), row(&[4, 3]), row(&[2, 1])];
    let opponent: Field = [row(&[6, 5]), row(&[4, 3]), row(&[2, 1])];
    assert_eq!(evaluate_winner(&player, &opponent), Outcome::Draw);
}

#[test]
fn normal_die_knockout_only_hits_unshielded_matching_faces() {
    let mine: Field = [row(&[3]), row(&[1, 1, 1]), row(&[])];
    let theirs: Field = [row(&[5, 5, 2]), row(&[]), srow(&[(5, true)])];
    let moves = legal_moves(&mine, &theirs, face(5), DieKind::Normal, false);

    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].placement(), Placement::mine(Row::ALL[0]));
    assert_eq!(moves[0].knockout(), &[0, 1]);
    assert_eq!(moves[1].placement(), Placement::mine(Row::ALL[2]));
    assert!(moves[1].knockout().is_empty());
}

#[test]
fn shielded_die_can_go_on_either_board_and_never_knocks() {
    let mine: Field = [row(&[]), row(&[1, 1, 1]), row(&[1, 1, 1])];
    let theirs: Field = [row(&[4]), row(&[]), row(&[2, 2, 2])];
    let moves = legal_moves(&mine, &theirs, face(4), DieKind::Shielded, false);

    let mine_n = moves
        .iter()
        .filter(|m| m.placement().board() == Board::Mine)
        .count();
    let their_n = moves
        .iter()
        .filter(|m| m.placement().board() == Board::Theirs)
        .count();
    assert_eq!(mine_n, 1);
    assert_eq!(their_n, 2);
    assert!(moves.iter().all(|m| m.knockout().is_empty()));
}

#[test]
fn first_die_is_own_board_only() {
    let empty: Field = [row(&[]), row(&[]), row(&[])];
    let moves = legal_moves(&empty, &empty, face(6), DieKind::Shielded, true);
    assert_eq!(moves.len(), 3);
    assert!(moves.iter().all(|m| m.placement().board() == Board::Mine));
}

#[test]
fn full_board_yields_no_moves() {
    let mine: Field = [row(&[1, 1, 1]), row(&[2, 2, 2]), row(&[3, 3, 3])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    assert!(legal_moves(&mine, &theirs, face(5), DieKind::Normal, false).is_empty());
}

#[test]
fn tiebreak_by_total_score() {
    let player: Field = [row(&[6, 6, 6]), row(&[5]), row(&[5])];
    let opponent: Field = [row(&[5]), row(&[6, 6, 6]), row(&[5])];
    assert_eq!(evaluate_winner(&player, &opponent), Outcome::Draw);

    let player2: Field = [row(&[6, 6, 6]), row(&[5]), row(&[6])];
    assert_eq!(evaluate_winner(&player2, &opponent), Outcome::Player);
}

#[test]
fn apply_move_places_in_first_empty_slot() {
    let mine: Field = [row(&[3]), row(&[2, 2]), row(&[])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    let mv = legal_moves(&mine, &theirs, face(4), DieKind::Normal, false)
        .into_iter()
        .find(|m| m.placement() == Placement::mine(Row::ALL[0]))
        .unwrap();
    let (new_mine, new_theirs) = apply_move(&mine, &theirs, die(4, false), &mv).unwrap();

    assert_eq!(new_mine[0][0], Some(die(3, false)));
    assert_eq!(new_mine[0][1], Some(die(4, false)));
    assert_eq!(new_mine[0][2], None);
    assert_eq!(new_mine[1], row(&[2, 2]));
    assert_eq!(new_mine[2], row(&[]));
    assert_eq!(new_theirs, theirs);
}

#[test]
fn apply_move_knockout_removes_only_specified_slots() {
    let mine: Field = [row(&[]), row(&[]), row(&[])];
    let theirs: Field = [row(&[5, 5, 2]), row(&[3]), row(&[])];
    let mv = legal_moves(&mine, &theirs, face(5), DieKind::Normal, false)
        .into_iter()
        .find(|m| m.placement() == Placement::mine(Row::ALL[0]))
        .unwrap();
    let (new_mine, new_theirs) = apply_move(&mine, &theirs, die(5, false), &mv).unwrap();

    assert_eq!(new_theirs[0][0], None);
    assert_eq!(new_theirs[0][1], None);
    assert_eq!(new_theirs[0][2], Some(die(2, false)));
    assert_eq!(new_theirs[1], theirs[1]);
    assert_eq!(new_theirs[2], theirs[2]);
    assert_eq!(new_mine[0][0], Some(die(5, false)));
}

#[test]
fn apply_move_shielded_on_theirs_scores_for_opponent() {
    let mine: Field = [row(&[]), row(&[]), row(&[])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    let mv = legal_moves(&mine, &theirs, face(6), DieKind::Shielded, false)
        .into_iter()
        .find(|m| m.placement() == Placement::theirs(Row::ALL[1]))
        .unwrap();
    let (new_mine, new_theirs) = apply_move(&mine, &theirs, die(6, true), &mv).unwrap();

    assert_eq!(new_theirs[1][0], Some(die(6, true)));
    assert_eq!(score_field(&new_theirs), 6);
    assert_eq!(score_field(&new_mine), 0);
}

#[test]
fn apply_move_rejects_move_reused_on_full_target_row() {
    let mine: Field = [row(&[]), row(&[]), row(&[])];
    let theirs: Field = [row(&[]), row(&[]), row(&[])];
    let mv = legal_moves(&mine, &theirs, face(1), DieKind::Normal, false)
        .into_iter()
        .find(|m| m.placement() == Placement::mine(Row::ALL[0]))
        .unwrap();
    let full_mine: Field = [row(&[1, 1, 1]), row(&[]), row(&[])];

    assert_eq!(
        apply_move(&full_mine, &theirs, die(1, false), &mv),
        Err(ApplyMoveError::NoEmptySlot {
            placement: Placement::mine(Row::ALL[0])
        })
    );
}
