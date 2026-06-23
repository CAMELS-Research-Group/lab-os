import { Link, Outlet, useLocation } from "react-router-dom";
import UpdateBanner from "./UpdateBanner";
import "./AppShell.css";

/**
 * Persistent app shell for the main (post-first-run) surface: a slim left icon
 * rail + a top context bar wrapping the routed screen via <Outlet />.
 *
 * The first-run route (Welcome) renders full-bleed and is intentionally NOT
 * wrapped by this shell.
 */

type NavItem = { to: string; label: string; icon: string; match: string[] };

const NAV_ITEMS: NavItem[] = [
  { to: "/home", label: "Home", icon: "◆", match: ["/home"] },
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
  const pathname = location.pathname;

  return (
    <div className="appshell">
      <nav className="rail" aria-label="Primary">
        {/* [DEFAULT]: brand glyph derived from the app name — replace after forking. */}
        <Link to="/home" className="rail-mark" aria-label="Home">
          [D]
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
            <span className="ctx-pill">
              <span className="ctx-pill-dot" aria-hidden="true" />
              On-device
            </span>
          </div>
        </header>

        <div className="appshell-scroll">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
