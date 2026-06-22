type Props = {
  /** Numeric series in the fixed [0,1] domain, oldest→newest. */
  series: number[];
  /** Stroke color (any CSS color, e.g. a `var(--token)`). */
  color: string;
  width?: number;
  height?: number;
};

/**
 * Small reusable inline-SVG sparkline. Plots `series` as a polyline over the
 * full width, with y mapped from the fixed [0,1] domain (the data is already a
 * rate). Higher values sit lower on screen so a downward line reads as "fewer
 * flags = better". A single-point (or empty) series renders a centered dot so
 * insufficient-data cards never crash.
 */
export default function Sparkline({
  series,
  color,
  width = 96,
  height = 28,
}: Props) {
  const pad = 2;
  const innerW = width - pad * 2;
  const innerH = height - pad * 2;

  // Map a [0,1] value to a y pixel; clamp out-of-range values for robustness.
  const yOf = (v: number) => {
    const clamped = Math.max(0, Math.min(1, v));
    return pad + clamped * innerH;
  };
  const xOf = (i: number, n: number) =>
    n <= 1 ? pad + innerW / 2 : pad + (i / (n - 1)) * innerW;

  const n = series.length;

  if (n === 0) {
    return (
      <svg
        className="sparkline"
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        aria-hidden="true"
      />
    );
  }

  if (n === 1) {
    return (
      <svg
        className="sparkline"
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        aria-hidden="true"
      >
        <circle cx={xOf(0, 1)} cy={yOf(series[0])} r={2.5} fill={color} />
      </svg>
    );
  }

  const points = series.map((v, i) => `${xOf(i, n)},${yOf(v)}`).join(" ");

  return (
    <svg
      className="sparkline"
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      aria-hidden="true"
    >
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth={1.75}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
