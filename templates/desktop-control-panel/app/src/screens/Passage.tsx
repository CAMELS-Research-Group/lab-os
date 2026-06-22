import { useNavigate } from "react-router-dom";
import { PASSAGE_PARAGRAPHS, PASSAGE_TITLE } from "../data/passage";
import "./Passage.css";

export default function Passage() {
  const nav = useNavigate();

  return (
    <div className="screen passage-screen">
      <h1>Today's reading</h1>
      <p className="lede">
        Read this passage to yourself and then read it aloud at a comfortable
        pace. When you're ready to speak, press <strong>Start</strong> and we'll
        listen.
      </p>

      <div className="card passage-card">
        <h2 className="passage-title">{PASSAGE_TITLE}</h2>
        {PASSAGE_PARAGRAPHS.map((p, i) => (
          <p key={i} className="passage-paragraph">
            {p}
          </p>
        ))}
      </div>

      <div className="passage-actions">
        <button className="primary" onClick={() => nav("/listening")}>
          Start
        </button>
      </div>
    </div>
  );
}
