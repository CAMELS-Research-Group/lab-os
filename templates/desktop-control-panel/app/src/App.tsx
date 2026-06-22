import { useEffect } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { useSession } from "./store/useSession";
import { useUpdate } from "./store/useUpdate";
import AppShell from "./components/AppShell";
import Welcome from "./screens/Welcome";
import Consent from "./screens/Consent";
import ModelDownload from "./screens/ModelDownload";
import L1Setup from "./screens/L1Setup";
import Home from "./screens/Home";
import Passage from "./screens/Passage";
import Listening from "./screens/Listening";
import Results from "./screens/Results";
import Progress from "./screens/Progress";
import Settings from "./screens/Settings";

function FirstRunGate() {
  const hasCompletedFirstRun = useSession((s) => s.hasCompletedFirstRun);
  const modelReady = useSession((s) => s.modelReady);

  // First launch: walk the first-run flow from the top (Welcome → Consent →
  // L1Setup). Once first-run is complete but the model hasn't downloaded yet,
  // route to the download screen. Only when both are satisfied does the gate
  // hand off to the Home dashboard inside the app shell.
  if (!hasCompletedFirstRun) return <Navigate to="/welcome" replace />;
  if (!modelReady) return <Navigate to="/model-download" replace />;
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
      <Route path="/consent" element={<Consent />} />
      <Route path="/model-download" element={<ModelDownload />} />
      <Route path="/setup" element={<L1Setup />} />

      {/* Main app — wrapped in the persistent icon-rail + context-bar shell. */}
      <Route element={<AppShell />}>
        <Route path="/home" element={<Home />} />
        <Route path="/passage" element={<Passage />} />
        <Route path="/listening" element={<Listening />} />
        <Route path="/results" element={<Results />} />
        <Route path="/progress" element={<Progress />} />
        <Route path="/settings" element={<Settings />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
