# PROJECT KNOWLEDGE BASE

**Generated:** 2026-07-01
**Commit:** cdc7725
**Branch:** main

## OVERVIEW

Windows helper for Lost Ark's Tikatuka minigame. It observes the game/window state and recommends the move most likely to help the human player win; keep game logic pure and testable before adding screen recognition.

## STRUCTURE

```text
tikatuka_guide/
├── Cargo.toml       # Rust crate (lib name `tikatuka`)
├── src/lib.rs       # crate re-exports only
├── src/core.rs      # core module re-exports
├── src/core/        # dice/field types + rules
├── src/search.rs    # search module internals + re-exports
├── src/search/      # public recommendation/advice API
├── src/*_tests.rs   # unit tests kept out of production modules
├── README.md        # placeholder
├── LICENSE          # MIT
└── AGENTS.md        # project memory
```

Only the pure rule engine and depth-limited solver exist so far. Vision and app
layers are still future. Add subdirectory AGENTS.md files only once a directory
has distinct conventions or enough code that local rules beat this root note.

## DOMAIN RULES

### 티카투카 규칙 (authoritative — per user, verbatim)

1. 나는 항상 왼쪽 보드, 상대는 항상 오른쪽 보드입니다.
2. 각 플레이어는 3줄을 가지고, 각 줄에는 주사위 3개까지 놓을 수 있습니다.
3. 줄 점수는 주사위 합계에 같은 숫자 보너스를 더합니다.
4. 3줄 중 2줄 이상 점수가 높으면 승리합니다.
5. 줄 승수가 같으면 모든 줄 총점을 합산해서 높은 쪽이 이깁니다.
6. 주사위는 일반 주사위와 실드 주사위가 있습니다.
7. 일반 주사위는 자기 보드에만 놓을 수 있습니다.
8. 실드 주사위는 자기 보드와 상대 보드 둘 다 놓을 수 있습니다.
9. 최초 시작 주사위는 무조건 실드이고, 현재 턴 플레이어 자기 보드에만 놓을 수 있습니다.
10. 일반 주사위 숫자가 상대 같은 줄의 일반 주사위 숫자와 같으면 알까기할 수 있습니다.
11. 알까기는 상대 해당 줄의 같은 숫자 일반 주사위를 전부 제거합니다.
12. 실드 주사위는 알까기로 제거되지 않습니다.
13. 알까기를 하려면 내 같은 줄에 빈칸이 있어야 합니다.
14. 내 같은 줄이 꽉 차 있으면 그 줄은 알까기할 수 없습니다.
15. 알까기 후에는 실드 주사위를 얻고, 그 실드 주사위는 다시 굴린 숫자를 입력해서 배치합니다.
16. 알까기로 얻은 실드 주사위는 자기 보드와 상대 보드 둘 다 놓을 수 있습니다.
17. 각 플레이어는 리롤권 1회를 가집니다.
18. 리롤하면 기존 숫자와 새 숫자 중 하나를 선택해서 둘 수 있습니다.
19. 한 플레이어의 보드가 꽉 차면 그 플레이어 턴은 자동으로 건너뜁니다.
20. 양쪽 보드가 모두 찼을 때 게임이 종료됩니다.

### Engine clarifications (precise reading of the above)

- **Board layout**: human = LEFT board, opponent = RIGHT board (rule 1). Rows are positionally aligned across boards (row 0 ↔ row 0).
- **Two die kinds** (rule 6): normal and shielded. Placement rights:
  - Normal die → own board only (rule 7).
  - Shielded die → either board (rule 8).
  - Exception: the game's first die is shielded but may only go on the current player's own board (rule 9).
- **Shield sources** (closed list): (a) the very first die of the game; (b) the die obtained after a successful knockout (rule 15). Shielded dice cannot be knocked out (rule 12) and shields do not affect scoring.
- **Knockout** (rules 10–14): placing a *normal* die whose face equals opponent *normal* dice in the *same aligned row* removes ALL opponent normal dice of that face in that row. Requires an empty slot in your own same row (rules 13–14) — i.e. you can only knock while placing into a non-full row. Shielded opponent dice are immune.
- **Post-knockout die** (rules 15–16): you gain a shielded die, input its rolled face, and place it on either board.
- **Reroll = "타짜의 손놀림"** (rules 17–18): once per game per player. Roll once more and keep either the old or the new face. (Same mechanic previously called Tazza hand movement.)
- **Scoring** (rule 3): row score = sum of faces + same-face bonus. n dice of the same face count as (2n−1) dice → contribute `face × (2n−1)`. Example: three 5s → 25; two 4s → 12.
- **Winner** (rules 4–5, confirmed): whoever wins MORE rows wins (tied rows count for neither) — 1 row-win beats 0 even with 2 draws. If row-wins are equal, higher total score (all rows summed) wins. If both equal, draw.
- **Game flow** (rules 19–20): a player whose board is full (9 dice) auto-skips their turn; the game ends when BOTH boards are full.

### Meta score (game score — separate from dice score)

- **Game score** is a persistent reward currency, distinct from the in-field dice score that decides the match. A normal win pays **+300**.
- **Tikatuka button**: a bet. Pressing "announces Tikatuka" and immediately costs **−200**. Winning the match while announced pays **+500** instead of +300. (Activation condition and payout sanity — see Open Questions.)

## PERCEPTION TARGETS

The vision layer must recognize these from the game screen/window and hand a
clean state to the core model (nothing else):

- The die the human just rolled and must place (current hand), and whether it is normal or shielded.
- Human field = LEFT board: all 3 rows, each slot's die face + shield flag.
- Opponent field = RIGHT board: all 3 rows, each slot's die face + shield flag.
- "Tazza hand movement" button — enabled/disabled state.
- "Tikatuka" button — enabled/disabled state.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Core types | `src/core/types.rs` | Dice faces, rows, placements, moves |
| Core rules, scoring, winner evaluation | `src/core/rules.rs` | Pure functions, no screen or UI dependencies |
| Move search and expected value | `src/search.rs` | Internal minimax/chance nodes |
| Public recommendation API | `src/search/advice.rs` | `TurnInput`, reroll advice, best placement |
| Screen/window recognition | future `src/vision/` | Convert pixels/window captures into core state only |
| Windows app, overlays, hotkeys | future `src/app/` | Presentation layer; never own game rules |
| Golden scenarios and self-checks | future `tests/` | Start with rule fixtures before CV tests |

## CODE MAP

Implemented under `src/` (crate `tikatuka`):

| Symbol | Type | Role |
|--------|------|------|
| `DieFace` | newtype | validated die face (1..=6); parse screen/OCR values at the boundary |
| `Die` | struct | one die; shield blocks knockout, not scoring |
| `Field = [[Option<Die>; 3]; 3]` | type | one player's board (3 rows × 3 slots) |
| `Row` | newtype | validated row index (0..3 internal rows) |
| `score_row` / `score_field` | fn | duplicate-count scoring (n same faces → 2n−1) |
| `evaluate_winner` | fn | more rows won, then total-score tiebreak, else draw |
| `DieKind`, `Board`, `Placement`, `Move` | types | placement-decision vocabulary |
| `legal_moves` | fn | legal placements for one die + flagged knockouts |
| `apply_move` | fn | apply a generated Move safely → `Result<(mine, theirs)>` |
| `heuristic_eval` | fn | shared leaf value (rows margin ×1000 + total margin), human POV |
| `recommend_move` | fn | best human placement, greedy one-ply |
| `Search` | struct | fixed (human, cpu) frame + per-player reroll flags threaded through the search |
| `turn_value` / `decide_roll` / `place_value` | fn | depth-limited search: chance (d6 avg) → optional reroll → placement; max(human)/min(cpu) |
| `bonus_value` | fn | post-knockout bonus shielded die (d6 avg, best own/opponent-board placement) then hand off |
| `recommend_move_search` | fn | best human placement via search (`DEFAULT_DEPTH = 3`) |
| `recommend_turn` / `TurnAdvice` | fn/enum | whether to reroll (타짜) or place — prices the reroll's option value |
| `recommend_after_reroll` / `RerollChoice` | fn/struct | after using 타짜, choose old/new face and placement |

Still future:

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| Tikatuka EV | fn | `src/search.rs` | meta-score bet advisor — blocked on open questions (activation + payout) |
| screen recognition | mod | future `src/vision/` | capture → `Field`/rolled die/button states |

## CONVENTIONS

- Model Tikatuka rules before UI. The solver should be runnable from fixtures without any Windows capture stack.
- Keep dice as explicit values plus shield state; do not infer shield from position after parsing.
- Keep recognition confidence separate from game state. A low-confidence screen read is not a legal game state.
- Prefer deterministic tests for rules and scoring. Random simulation belongs behind an injected RNG/seed.
- Use Korean game terms in user-facing copy where useful, but keep code identifiers English and boring.

## ANTI-PATTERNS

- Do not mix screen scraping with rule evaluation.
- Do not hardcode coordinates until the capture target and Lost Ark window scaling strategy are chosen.
- Do not add a custom cache, plugin system, or strategy framework before a single solver path exists.
- Do not automate game input unless the user explicitly asks and the risk is reviewed separately.

## COMMANDS

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Keep UI/CV verification separate from these pure-model checks.

## NOTES

- `README.md` is currently only the repository title.
- `.omc/` and `.codegraph/` are local tool artifacts, not project structure.
- The "score difference" tiebreak should be confirmed against observed in-game results once fixtures are available.

## OPEN QUESTIONS

Confirm with the user before implementing logic that depends on these:

1. **Tikatuka activation condition** — when the button becomes enabled/disabled is unknown (user to add). Perception can read enabled/disabled state now; the advisor needs the rule.
2. **Tikatuka payout numbers need confirming** — as stated, pressing seems never worth it: a win while announced nets +500 − 200 = **+300**, identical to a normal win (+300), while a loss costs **−200** (vs 0). So the literal numbers make "never press" always optimal, which defeats the planned advisor. Likely one of these is off — e.g. the −200 is refunded on a win, the announced-win reward is higher than 500, or a normal win pays less than 300. Confirm before building the press/no-press EV advisor.

Resolved (folded into DOMAIN RULES): full shield conditions (closed list of two — first die of the game; post-knockout reroll); placement row rules (any non-full row, free choice); score-diff tiebreak (total across all rows); first-player is random with no effect beyond the shielded first die.
