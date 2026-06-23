import "./Home.css";

function greeting(hour: number): string {
  if (hour < 12) return "Good morning";
  if (hour < 18) return "Good afternoon";
  return "Good evening";
}

/**
 * Home dashboard — the calm landing space for the main app shell. A generic
 * greeting and an on-device privacy note. Application-specific content is added
 * by whatever product is built on top of this control-panel template.
 */
export default function Home() {
  return (
    <div className="screen home-screen">
      <div className="home-center">
        <div className="home-eyebrow">{greeting(new Date().getHours())}</div>
        <h1 className="home-head">
          Welcome
          <br />
          <em>back.</em>
        </h1>
        <p className="lede">
          Everything here runs on this device. Use the rail to move between Home
          and Settings.
        </p>
      </div>
    </div>
  );
}
