/**
 * Derived, on-device practice stats for the Home dashboard and the shell's
 * context-bar mini-ring (app-shell Option 2).
 *
 * PROVISIONAL — the "weekly goal" target and what the ring measures are
 * IAS-owned pedagogy (per CLAUDE.md; not an engineering decision). Until IAS
 * sets the real goal, the ring shows sessions completed in the last 7 days
 * against a placeholder target. Everything here is corpus-level aggregate
 * derived from already-persisted session/evaluation state — no raw audio,
 * transcripts, or PII (privacy posture, CLAUDE.md).
 */

import type { EvaluationResult } from "../ipc/types";
import type { SessionSummary } from "../store/useSession";

/** Placeholder weekly session target until IAS defines the real goal. */
export const WEEKLY_SESSION_GOAL = 5;

const WEEK_MS = 7 * 24 * 60 * 60 * 1000;

/** Sessions whose `ended_at` falls within the trailing 7-day window. */
export function sessionsThisWeek(
  sessions: SessionSummary[],
  now: number = Date.now()
): number {
  const cutoff = now - WEEK_MS;
  return sessions.filter((s) => {
    const t = Date.parse(s.ended_at);
    return Number.isFinite(t) && t >= cutoff;
  }).length;
}

/**
 * Provisional weekly-goal completion as a 0–100 integer, for the conic-gradient
 * rings. Clamped at 100; 0 when there is no history yet.
 */
export function weeklyGoalPercent(
  sessions: SessionSummary[],
  now: number = Date.now()
): number {
  if (WEEKLY_SESSION_GOAL <= 0) return 0;
  const pct = (sessionsThisWeek(sessions, now) / WEEKLY_SESSION_GOAL) * 100;
  return Math.max(0, Math.min(100, Math.round(pct)));
}

/** Single derived stat for a Home stat card. `value` is display-ready. */
export type HomeStat = { value: string; label: string; emphasis?: string };

/**
 * The clearest phoneme from the most recent evaluation — the one with the
 * highest mean certainty among phonemes that actually occurred. Null when there
 * is no evaluation yet or none had a real certainty score.
 */
export function clearestPhoneme(
  evaluation: EvaluationResult | null
): string | null {
  if (!evaluation) return null;
  let best: { phoneme: string; certainty: number } | null = null;
  for (const [phoneme, rollup] of Object.entries(evaluation.phoneme_attempts)) {
    if (rollup.mean_certainty === null || rollup.occurrences === 0) continue;
    if (!best || rollup.mean_certainty > best.certainty) {
      best = { phoneme, certainty: rollup.mean_certainty };
    }
  }
  return best ? best.phoneme : null;
}

/** The flagged phonemes from the most recent evaluation, in priority order. */
export function flaggedPhonemes(evaluation: EvaluationResult | null): string[] {
  if (!evaluation) return [];
  return evaluation.flagged_phonemes_ordered.map((f) => f.phoneme);
}
