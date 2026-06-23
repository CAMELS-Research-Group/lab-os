import { useEffect } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { useSession } from "./store/useSession";
import { useUpdate } from "./store/useUpdate";
import AppShell from "./components/AppShell";
import Welcome from "./screens/Welcome";
import Home from "./screens/Home";
import Settings from "./screens/Settings";

function FirstRunGate() {
  const hasCompletedFirstRun = useSession((s) => s.hasCompletedFirstRun);

  // First launch: route to the Welcome gate. Once first-run is complete, hand
  // off to the Home dashboard inside the app shell.
  if (!hasCompletedFirstRun) return <Navigate to="/welcome" replace />;
  return <Navigate to="/home" replace />;
}

export default function App() {
  const check = useUpdate((s) => s.check);

  // Fire the update check when the app shell mounts. Empty-deps useEffect, so
  // it does not re-run on navigation. Note: under React 18 StrictMode in dev
  // the effect runs twice on mount — harmless here, since each call reaches the
  // Rust gate and is a no-op when update checks are disabled. `check` is a
  // stable Zustand action reference, so it is intentionally omitted from deps.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => { void check(); }, []);

  return (
    <Routes>
      <Route path="/" element={<FirstRunGate />} />

      {/* First-run flow — full-bleed, outside the app shell. */}
      <Route path="/welcome" element={<Welcome />} />

      {/* Main app — wrapped in the persistent icon-rail + context-bar shell. */}
      <Route element={<AppShell />}>
        <Route path="/home" element={<Home />} />
        <Route path="/settings" element={<Settings />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
