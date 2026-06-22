import { useState, type CSSProperties } from "react";
import { Link, Outlet, useLocation } from "react-router-dom";
import { useSession } from "../store/useSession";
import { weeklyGoalPercent } from "../lib/stats";
import UpdateBanner from "./UpdateBanner";
import ShareModal from "./ShareModal";
import "./AppShell.css";

/**
 * Persistent app shell for the main (post-first-run) surface: a slim left icon
 * rail + a top context bar wrapping the routed screen via <Outlet />. This is
 * the "quiet & focused" (Option 2) direction — one calm space rather than a
 * deck of separate screens.
 *
 * First-run routes (Welcome / Consent / ModelDownload / L1Setup) render
 * full-bleed and are intentionally NOT wrapped by this shell.
 *
 * The context bar absorbs the controls that used to live in the floating
 * AppHeader (Give Feedback + Settings), so AppHeader is retired.
 */

type NavItem = { to: string; label: string; icon: string; match: string[] };

// Top-of-rail destinations. The "Read" item owns the whole practice flow
// (passage → listening → results), so its active state spans those routes.
const NAV_ITEMS: NavItem[] = [
  { to: "/home", label: "Home", icon: "◆", match: ["/home"] },
  { to: "/passage", label: "Read", icon: "▶", match: ["/passage", "/listening", "/results"] },
  { to: "/progress", label: "Progress", icon: "▤", match: ["/progress"] },
];

const SETTINGS_ITEM: NavItem = {
  to: "/settings",
  label: "Settings",
  icon: "⚙",
  match: ["/settings"],
};

function isActive(item: NavItem, pathname: string): boolean {
  return item.match.some((m) => pathname === m || pathname.startsWith(m + "/"));
}

function titleFor(pathname: string): string {
  if (isActive(NAV_ITEMS[0], pathname)) return "Home";
  if (pathname.startsWith("/passage") || pathname.startsWith("/listening")) {
    return "Read aloud";
  }
  if (pathname.startsWith("/results")) return "Results";
  if (pathname.startsWith("/progress")) return "Progress";
  if (pathname.startsWith("/settings")) return "Settings";
  return "";
}

function RailLink({ item, pathname }: { item: NavItem; pathname: string }) {
  const active = isActive(item, pathname);
  return (
    <Link
      to={item.to}
      className={"rail-link" + (active ? " rail-link--on" : "")}
      aria-label={item.label}
      aria-current={active ? "page" : undefined}
    >
      <span className="rail-icon" aria-hidden="true">
        {item.icon}
      </span>
      <span className="rail-tip">{item.label}</span>
    </Link>
  );
}

export default function AppShell() {
  const location = useLocation();
  const sessions = useSession((s) => s.sessions);
  const [feedbackOpen, setFeedbackOpen] = useState(false);

  const pathname = location.pathname;
  const pct = weeklyGoalPercent(sessions);

  return (
    <div className="appshell">
      <nav className="rail" aria-label="Primary">
        <Link to="/home" className="rail-mark" aria-label="Home">
          P
        </Link>
        {NAV_ITEMS.map((item) => (
          <RailLink key={item.to} item={item} pathname={pathname} />
        ))}
        <div className="rail-spacer" />
        <RailLink item={SETTINGS_ITEM} pathname={pathname} />
      </nav>

      <div className="appshell-col">
        <UpdateBanner />
        <header className="ctx">
          <h2 className="ctx-title">{titleFor(pathname)}</h2>
          <div className="ctx-right">
            <button
              className="ctx-feedback"
              onClick={() => setFeedbackOpen(true)}
            >
              Give Feedback
            </button>
            <span className="ctx-pill">
              <span className="ctx-pill-dot" aria-hidden="true" />
              On-device
            </span>
            <span className="ctx-week">This week</span>
            <span
              className="ctx-miniring"
              style={{ "--p": pct } as CSSProperties}
              title={`Weekly goal (provisional): ${pct}%`}
            >
              <i>{pct}%</i>
            </span>
          </div>
        </header>

        <div className="appshell-scroll">
          <Outlet />
        </div>
      </div>

      <ShareModal open={feedbackOpen} onClose={() => setFeedbackOpen(false)} />
    </div>
  );
}
