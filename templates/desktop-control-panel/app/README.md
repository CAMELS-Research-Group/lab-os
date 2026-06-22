# P3 Platform — Walking-Skeleton Demo

Throwaway-but-credible Tauri shell that walks the five V1 screens with **real
microphone capture and faked phoneme inference**. Built to drive user-flow
conversations at the next IAS stakeholder meeting before committing to ADD-level
decisions and real inference work.

This is **not the production app**. It exists to make the V1 user experience
tangible. Everything that would normally come back from the encoder is scripted
in `src/data/scriptedResults.ts`.

## Running

Prerequisites: Node 20+, npm, and the Rust toolchain (`rustup` + MSVC build
tools on Windows).

```bash
cd app
npm install      # one time
npm run tauri dev
```

First launch builds the Tauri Rust shell, which can take 3–8 minutes while
Cargo pulls dependencies. Subsequent launches are seconds.

For a faster iteration loop on the React UI alone (no Tauri shell), run
`npm run dev` and open `http://localhost:1420` in a browser.

## The demo storyline

- Learner: "Mei", L1 Mandarin.
- She picks her L1 in setup, reads the visiting_nyc.txt passage, sees a few
  non-blocking pings during the read, gets a per-phoneme summary flagging
  /θ/, /r/, /l/, taps /θ/ to read articulatory guidance, then sees a Progress
  view with four prior sessions and trend arrows. She decides to share with
  her tutor and sees exactly what would leave the device.

The fake history, scripted results, and ping schedule are constants so the
demo is reproducible across stakeholder meetings.

## What's fake

- All inference. Per-phoneme attempt and flag counts come from
  `SCRIPTED_RESULTS` regardless of what's said into the mic.
- The 4 prior sessions. Constants in `fakeHistory.ts`.
- The "ping" events during reading. Scheduled in `pingSchedule.ts` by absolute
  timestamp from "Start" press; not actually triggered by speech.
- The "Send to tutor" action — the modal is a no-op confirmation.

## What's real

- Microphone capture via `navigator.mediaDevices.getUserMedia` and
  `MediaRecorder`. Stakeholders can play back what they recorded on the
  results screen.
- The L1 questionnaire and its persistence to `localStorage`.
- The articulation-guidance copy — drawn verbatim from
  `documentation/docs/pedagogy/articulation_table.md` (draft v0.1, pending
  phonetician review).
- The 13 target phonemes, the passage, and the FRD-specified L1 suggestions.

## Reset

A small **Reset** button in the bottom-right corner clears localStorage and
returns to the first-run flow — for dev use during demos.

## Out of scope

Called out in the parent plan
(`.claude/plans/my-next-task-is-precious-harp.md`). Worth reviewing before the
meeting so out-of-scope items don't get treated as feedback gaps:

- Real phoneme inference (WavLM Base+ CTC head, `camels` package).
- Threshold calibration with IAS tutor scoring.
- Phonetician sign-off on articulation copy.
- IPA sagittal diagrams (CC BY-SA 3.0 sourcing).
- Installer / signing / model download.
- SQLite history persistence.
- Accessibility audit.
