import { useEffect, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { getSessionHistory } from "../ipc/commands";
import { useSession } from "../store/useSession";
import { TARGET_PHONEMES } from "../data/phonemes";
import {
  weeklyGoalPercent,
  clearestPhoneme,
  flaggedPhonemes,
} from "../lib/stats";
import "./Home.css";

function greeting(hour: number): string {
  if (hour < 12) return "Good morning";
  if (hour < 18) return "Good afternoon";
  return "Good evening";
}

/**
 * Home dashboard — the calm landing space for the main app (app-shell Option 2).
 * One centered column: a greeting, a "Ready to practice?" start block with the
 * (provisional) weekly-goal ring + CTA into the practice flow, and a row of
 * derived stat cards. All stats are corpus-level aggregates from already-stored
 * session/evaluation state — no raw audio, transcripts, or PII.
 */
export default function Home() {
  const nav = useNavigate();
  const sessions = useSession((s) => s.sessions);
  const setSessions = useSession((s) => s.setSessions);
  const lastEvaluation = useSession((s) => s.lastEvaluation);

  // Keep the dashboard counts honest with on-disk history (same fetch the
  // Progress screen uses). Failures are non-fatal — fall back to whatever is
  // already in the store.
  useEffect(() => {
    getSessionHistory()
      .then((r) =>
        setSessions(
          r.sessions.map((s) => ({
            session_id: s.session_id,
            ended_at: s.ended_at,
            duration_seconds: s.duration_seconds,
          }))
        )
      )
      .catch(() => {});
  }, [setSessions]);

  const pct = weeklyGoalPercent(sessions);
  const clearest = clearestPhoneme(lastEvaluation);
  const flagged = flaggedPhonemes(lastEvaluation);
  const hasHistory = sessions.length > 0;

  return (
    <div className="screen home-screen">
      <div className="home-center">
        <div className="home-eyebrow">{greeting(new Date().getHours())}</div>
        <h1 className="home-head">
          Let's clear up
          <br />
          <em>a few sounds.</em>
        </h1>
        <p className="lede">
          A short reading is all it takes. We'll show which of your{" "}
          {TARGET_PHONEMES.length} target sounds came through clearly.
        </p>

        <div className="home-startblock">
          <div
            className="home-bigring"
            style={{ "--p": pct } as CSSProperties}
            title="Weekly goal (provisional)"
          >
            <i>
              <b>{pct}%</b>
              <small>WEEKLY GOAL</small>
            </i>
          </div>
          <div className="home-startbody">
            <h3>Ready to practice?</h3>
            <p>
              Read a passage aloud — it takes a few minutes, and everything stays
              on this device.
            </p>
            <button className="home-cta primary" onClick={() => nav("/passage")}>
              ▶ Start reading
            </button>
          </div>
        </div>

        <div className="home-stats">
          <div className="home-stat">
            <div className="home-stat-n">{sessions.length}</div>
            <div className="home-stat-k">
              {sessions.length === 1 ? "session" : "sessions"}
            </div>
          </div>
          <div className="home-stat">
            <div className="home-stat-n ipa">{clearest ?? "—"}</div>
            <div className="home-stat-k">clearest</div>
          </div>
          <div className="home-stat">
            <div className="home-stat-n">{flagged.length}</div>
            <div className="home-stat-k">
              to focus
              {flagged.length > 0 && (
                <>
                  {" · "}
                  <b className="ipa">{flagged.slice(0, 3).join(" ")}</b>
                </>
              )}
            </div>
          </div>
        </div>

        {!hasHistory && (
          <p className="home-firsttime">
            This is your first session — your stats fill in as you practice.
          </p>
        )}
      </div>
    </div>
  );
}
