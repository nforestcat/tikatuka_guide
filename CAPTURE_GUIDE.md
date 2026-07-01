# Continuity guide — capturing a Tikatuka frame on the Lost Ark machine

The pure rule engine + full search are done. The next step (the screen
recognizer) needs a **real Tikatuka screenshot** to calibrate. This guide is for
capturing one on the machine that runs Lost Ark.

## 1. Get the code
```bash
git clone https://github.com/nforestcat/tikatuka_guide.git
cd tikatuka_guide
# or, if already cloned:  git pull
```

## 2. Prerequisites
- Rust toolchain — install from https://rustup.rs if `cargo` is missing.
- Build (first build downloads deps, ~1 min):
```bash
cargo build
```

## 3. Capture a frame
- Launch Lost Ark → enter a Tikatuka game → reach a **mid-game** state.
- Find the game window's title:
```bash
cargo run --bin capture -- --list
```
- Capture it (replace `lost` with a distinctive substring of the game window
  title shown in the list):
```bash
cargo run --bin capture -- lost shots/frame1.png
```

## 4. What to capture (2–4 PNGs covering these is ideal)
- A mid-game frame with **both 3×3 boards** partly filled and the **rolled die** visible.
- A frame showing a **shielded die** (its shield marker) next to normal dice.
- The **Tazza (타짜)** and **Tikatuka** buttons — ideally one frame with them
  enabled and one disabled.
- Whatever shows how dice render (pips vs. numbers) — one clear frame answers it.

⚠️ **If the saved PNG is black** (some DirectX games block window capture): run
Lost Ark in **borderless/windowed** mode and retry, or use **Win+Shift+S**
(Snipping Tool) and save the region as a PNG. Any clear screenshot works — the
`capture` tool is only a convenience.

## 5. Send the frames back
Commit them to the repo:
```bash
git add shots && git commit -m "Add Tikatuka reference screenshots" && git push
```
Then in the next session, say they're in `shots/` and I'll build the recognizer.
(Or just drag a PNG straight into the chat.)

## Status
- **Done & pushed:** rule engine + expectimax search (knockout bonus die +
  reroll), reviewed; `capture` tool.
- **Next (needs a screenshot):** frame → `TurnInput` recognizer, then the
  capture → recognize → `recommend_turn` app loop.
- **Blocked on you (rules, not machines):** the Tikatuka button's activation
  condition + payout structure, for the meta-score EV advisor. See the OPEN
  QUESTIONS section in `AGENTS.md`.
