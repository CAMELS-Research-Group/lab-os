import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { getSessionHistory, getPhonemeTrends } from "../ipc/commands";
import type { PhonemeTrend, TrendDirection } from "../ipc/types";
import { useSession } from "../store/useSession";
import PhonemeBadge from "../components/PhonemeBadge";
import Sparkline from "../components/Sparkline";
import "./Progress.css";

function fmtDateTime(iso: string) {
  const d = new Date(iso);
  const datePart = d.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
  const timePart = d.toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
  return `${datePart} · ${timePart}`;
}

function fmtDuration(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

// Minimum observed sessions before a phoneme gets a ranked trend card.
// Mirror of the `n < 2` guard in `trend_direction` (src-tauri/src/storage/
// commands.rs): below this count Rust returns Flat, so a future edit to either
// "2 observed sessions" threshold must update the other to stay consistent.
const MIN_OBSERVED = 2;

type TrendMeta = {
  label: string;
  arrow: string;
  color: string; // CSS var() expression
  rank: number; // sort key: lower renders first
};

// Settled mapping (do not re-litigate): worsening → "Needs work"/--warn first,
// flat → "Steady"/--ink-faint, improving → "Improving"/--good last.
const TREND_META: Record<TrendDirection, TrendMeta> = {
  worsening: { label: "Needs work", arrow: "↓", color: "var(--warn)", rank: 0 },
  flat: { label: "Steady", arrow: "→", color: "var(--ink-faint)", rank: 1 },
  improving: {
    label: "Improving",
    arrow: "↑",
    color: "var(--good)",
    rank: 2,
  },
};

function TrendCard({ trend }: { trend: PhonemeTrend }) {
  const meta = TREND_META[trend.trend_direction];
  return (
    <div className="trend-card">
      <div className="trend-card-head">
        <PhonemeBadge ipa={trend.phoneme} size="lg" />
        <span className="trend-example">{trend.example_word}</span>
      </div>
      <Sparkline series={trend.session_flag_rate} color={meta.color} />
      <div className="trend-pill" style={{ color: meta.color }}>
        <span className="trend-arrow" aria-hidden="true">
          {meta.arrow}
        </span>
        <span>{meta.label}</span>
      </div>
    </div>
  );
}

function InsufficientCard({ trend }: { trend: PhonemeTrend }) {
  return (
    <div className="trend-card trend-card-quiet">
      <div className="trend-card-head">
        <PhonemeBadge ipa={trend.phoneme} size="lg" />
        <span className="trend-example">{trend.example_word}</span>
      </div>
      <p className="trend-insufficient">Not enough practice yet</p>
    </div>
  );
}

export default function Progress() {
  const nav = useNavigate();
  const sessions = useSession((s) => s.sessions);
  const setSessions = useSession((s) => s.setSessions);

  // Trends are not persisted — local component state only (independent fetch).
  const [trends, setTrends] = useState<PhonemeTrend[]>([]);

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

  useEffect(() => {
    getPhonemeTrends()
      .then((r) => setTrends(r.per_phoneme))
      .catch(() => setTrends([]));
  }, []);

  // Store is oldest-first; render most recent first.
  const orderedSessions = [...sessions].reverse();

  // Ranked cards: enough observed sessions, sorted needs-work → steady →
  // improving. Insufficient cards trail after, grouped together.
  const ranked = trends
    .filter((t) => t.sessions_observed >= MIN_OBSERVED)
    .sort(
      (a, b) =>
        TREND_META[a.trend_direction].rank - TREND_META[b.trend_direction].rank
    );
  const insufficient = trends.filter(
    (t) => t.sessions_observed < MIN_OBSERVED
  );

  const hasTrends = ranked.length > 0 || insufficient.length > 0;

  return (
    <div className="screen progress-screen">
      <h1>Your progress</h1>
      {sessions.length === 0 ? (
        <>
          <p className="lede">
            Complete a session to start tracking your practice. Each reading is
            saved on this device, and you'll get per-sound feedback right after
            every session.
          </p>
          <div className="card empty-state">
            <p>
              No sessions yet. Tap <strong>Practice again</strong> to record
              your first.
            </p>
          </div>
        </>
      ) : (
        <>
          <p className="lede">
            Your past readings, most recent first, with per-sound trends across
            your sessions.
          </p>

          {hasTrends && (
            <div className="trend-grid">
              {ranked.map((t) => (
                <TrendCard key={t.phoneme} trend={t} />
              ))}
              {insufficient.map((t) => (
                <InsufficientCard key={t.phoneme} trend={t} />
              ))}
            </div>
          )}

          <div className="card sessions-card">
            <div className="sessions-card-label">Recent sessions</div>
            <ul className="session-list">
              {orderedSessions.map((s) => (
                <li key={s.session_id} className="session-row">
                  <span className="session-date">
                    {fmtDateTime(s.ended_at)}
                  </span>
                  <span className="session-duration">
                    {fmtDuration(s.duration_seconds)}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </>
      )}

      <div className="progress-actions">
        <button className="primary" onClick={() => nav("/passage")}>
          Practice again
        </button>
      </div>
    </div>
  );
}
