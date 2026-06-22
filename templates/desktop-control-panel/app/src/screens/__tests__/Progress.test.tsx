/**
 * Unit tests for the Progress screen.
 *
 * Two concerns covered:
 *
 *  1. Session history (#117 regression guard): Progress reads persisted
 *     sessions from SQLite via `getSessionHistory()` on mount and renders the
 *     compact recent-sessions list. Errors degrade to the empty state.
 *
 *  2. Per-phoneme trend cards (#147): Progress also fetches
 *     `getPhonemeTrends()` into local state and renders one card per phoneme —
 *     a big IPA glyph, example word, a sparkline, and an Improving / Steady /
 *     Needs work pill mapped from `trend_direction`. Cards sort needs-work →
 *     steady → improving; phonemes with `sessions_observed < 2` group after as
 *     quiet "not enough practice yet" cards. A trends-fetch rejection degrades
 *     gracefully (recent-sessions list still renders, no crash).
 *
 * Both fetches are independent: every test mocks BOTH commands.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import Progress from "../Progress";
import { useSession } from "../../store/useSession";
import type { PhonemeTrend } from "../../ipc/types";

// ---------------------------------------------------------------------------
// Module mock — hoisted by vitest so the mock is in place before any import.
// ---------------------------------------------------------------------------

const { mockGetSessionHistory, mockGetPhonemeTrends } = vi.hoisted(() => ({
  mockGetSessionHistory: vi.fn(),
  mockGetPhonemeTrends: vi.fn(),
}));

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return {
    ...real,
    getSessionHistory: mockGetSessionHistory,
    getPhonemeTrends: mockGetPhonemeTrends,
  };
});

// ---------------------------------------------------------------------------
// Reset helpers + defaults
// ---------------------------------------------------------------------------

beforeEach(() => {
  useSession.setState({ sessions: [] });
  vi.clearAllMocks();
  // Sensible defaults — individual tests override as needed.
  mockGetSessionHistory.mockResolvedValue({ sessions: [] });
  mockGetPhonemeTrends.mockResolvedValue({ per_phoneme: [] });
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function renderProgress() {
  return render(
    <MemoryRouter>
      <Progress />
    </MemoryRouter>
  );
}

const oneSession = {
  session_id: "s1",
  ended_at: "2026-06-09T12:00:00Z",
  duration_seconds: 120,
  flagged_count: 2,
  highest_error_phoneme: "θ",
};

function trend(over: Partial<PhonemeTrend>): PhonemeTrend {
  return {
    phoneme: "θ",
    example_word: "think",
    attempts_total: 10,
    flagged_total: 3,
    trend_direction: "flat",
    sessions_observed: 3,
    session_flag_rate: [0.4, 0.3, 0.3],
    ...over,
  };
}

// ---------------------------------------------------------------------------
// Tests — session history
// ---------------------------------------------------------------------------

describe("Progress screen — session history", () => {
  it("renders a session row after getSessionHistory resolves", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [oneSession] });

    renderProgress();

    const durationCell = await screen.findByText("2:00");
    expect(durationCell).toBeInTheDocument();
  });

  it("swallows getSessionHistory errors and shows empty-state copy", async () => {
    mockGetSessionHistory.mockRejectedValue(new Error("no tauri backend"));

    renderProgress();

    const emptyMsg = await screen.findByText(/no sessions yet/i);
    expect(emptyMsg).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Tests — per-phoneme trend cards
// ---------------------------------------------------------------------------

describe("Progress screen — per-phoneme trend cards", () => {
  it("renders the three trend states with their pill copy and example words", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [oneSession] });
    mockGetPhonemeTrends.mockResolvedValue({
      per_phoneme: [
        trend({
          phoneme: "θ",
          example_word: "think",
          trend_direction: "worsening",
        }),
        trend({
          phoneme: "ɹ",
          example_word: "red",
          trend_direction: "flat",
        }),
        trend({
          phoneme: "v",
          example_word: "very",
          trend_direction: "improving",
        }),
      ],
    });

    renderProgress();

    expect(await screen.findByText("Needs work")).toBeInTheDocument();
    expect(screen.getByText("Steady")).toBeInTheDocument();
    expect(screen.getByText("Improving")).toBeInTheDocument();
    expect(screen.getByText("think")).toBeInTheDocument();
    expect(screen.getByText("red")).toBeInTheDocument();
    expect(screen.getByText("very")).toBeInTheDocument();
  });

  it("sorts ranked cards needs-work → steady → improving", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [oneSession] });
    mockGetPhonemeTrends.mockResolvedValue({
      per_phoneme: [
        // Supplied improving-first to prove the component re-sorts.
        trend({ phoneme: "v", trend_direction: "improving" }),
        trend({ phoneme: "ɹ", trend_direction: "flat" }),
        trend({ phoneme: "θ", trend_direction: "worsening" }),
      ],
    });

    renderProgress();

    await screen.findByText("Needs work");
    const pills = screen
      .getAllByText(/Needs work|Steady|Improving/)
      .map((el) => el.textContent);
    expect(pills).toEqual(["Needs work", "Steady", "Improving"]);
  });

  it("groups insufficient-data phonemes after ranked cards", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [oneSession] });
    mockGetPhonemeTrends.mockResolvedValue({
      per_phoneme: [
        trend({
          phoneme: "θ",
          example_word: "think",
          trend_direction: "worsening",
          sessions_observed: 3,
        }),
        trend({
          phoneme: "ʒ",
          example_word: "measure",
          trend_direction: "flat",
          sessions_observed: 1,
          session_flag_rate: [0.5],
        }),
      ],
    });

    renderProgress();

    const notEnough = await screen.findByText(/not enough practice yet/i);
    expect(notEnough).toBeInTheDocument();
    // The insufficient card carries its phoneme's example word but no pill.
    expect(screen.getByText("measure")).toBeInTheDocument();
    expect(screen.queryByText("Steady")).not.toBeInTheDocument();
    // Ranked needs-work card still present.
    expect(screen.getByText("Needs work")).toBeInTheDocument();
  });

  it("shows the empty state and no trend cards when there are no sessions", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [] });
    mockGetPhonemeTrends.mockResolvedValue({
      per_phoneme: [trend({ trend_direction: "worsening" })],
    });

    renderProgress();

    const emptyMsg = await screen.findByText(/no sessions yet/i);
    expect(emptyMsg).toBeInTheDocument();
    expect(screen.queryByText("Needs work")).not.toBeInTheDocument();
  });

  it("degrades gracefully when getPhonemeTrends rejects", async () => {
    mockGetSessionHistory.mockResolvedValue({ sessions: [oneSession] });
    mockGetPhonemeTrends.mockRejectedValue(new Error("no trends handler"));

    renderProgress();

    // Recent-sessions list still renders (duration visible), no trend cards.
    const recent = await screen.findByText("Recent sessions");
    expect(recent).toBeInTheDocument();
    expect(within(recent.closest(".sessions-card")!).getByText("2:00"))
      .toBeInTheDocument();
    expect(screen.queryByText("Needs work")).not.toBeInTheDocument();
  });
});
