// V1 target phoneme inventory. All learner-facing articulation copy (mouth
// shape, example word, minimal pair) lives in the single source of truth —
// documentation/docs/pedagogy/articulation_table.md — and reaches the UI via
// the Rust `FeedbackEntry`, not this file.
export const TARGET_PHONEMES: readonly string[] = [
  "r",
  "l",
  "v",
  "w",
  "θ",
  "ð",
  "ʒ",
  "dʒ",
  "z",
  "i",
  "ɪ",
  "ɛ",
  "æ",
] as const;
