// Thin MediaRecorder wrapper for the "Listen back" playback capture (#116).
// Runs alongside — and independent of — the Rust evaluation capture; the blob
// is exposed only as an in-memory object URL for an <audio> element on Results,
// never written to disk or localStorage and never transmitted (FRD F-HIS-3 /
// NF-PRV-2). Permission denial does not block the flow: the listening screen
// falls back to a "simulated recording" badge and continues.

export type RecorderState = "idle" | "recording" | "paused" | "stopped" | "denied";

export type RecorderResult = {
  blob: Blob;
  objectUrl: string;
  durationMs: number;
};

export class Recorder {
  private mediaRecorder: MediaRecorder | null = null;
  private chunks: BlobPart[] = [];
  private stream: MediaStream | null = null;
  private startTime = 0;
  private accumulatedMs = 0;
  state: RecorderState = "idle";

  async start(): Promise<void> {
    try {
      this.stream = await navigator.mediaDevices.getUserMedia({
        audio: { channelCount: 1, sampleRate: 48000 },
      });
    } catch {
      this.state = "denied";
      throw new Error("microphone permission denied");
    }
    this.chunks = [];
    this.accumulatedMs = 0;
    this.mediaRecorder = new MediaRecorder(this.stream);
    this.mediaRecorder.ondataavailable = (e) => {
      if (e.data && e.data.size > 0) this.chunks.push(e.data);
    };
    this.mediaRecorder.start(100);
    this.startTime = performance.now();
    this.state = "recording";
  }

  pause(): void {
    if (!this.mediaRecorder || this.state !== "recording") return;
    this.mediaRecorder.pause();
    this.accumulatedMs += performance.now() - this.startTime;
    this.state = "paused";
  }

  resume(): void {
    if (!this.mediaRecorder || this.state !== "paused") return;
    this.mediaRecorder.resume();
    this.startTime = performance.now();
    this.state = "recording";
  }

  async stop(): Promise<RecorderResult> {
    return new Promise((resolve) => {
      if (!this.mediaRecorder) {
        resolve({
          blob: new Blob([]),
          objectUrl: "",
          durationMs: 0,
        });
        return;
      }
      const finalize = () => {
        if (this.state === "recording") {
          this.accumulatedMs += performance.now() - this.startTime;
        }
        const blob = new Blob(this.chunks, {
          type: this.mediaRecorder?.mimeType || "audio/webm",
        });
        const objectUrl = URL.createObjectURL(blob);
        this.stream?.getTracks().forEach((t) => t.stop());
        this.stream = null;
        this.state = "stopped";
        resolve({ blob, objectUrl, durationMs: this.accumulatedMs });
      };
      this.mediaRecorder.addEventListener("stop", finalize, { once: true });
      this.mediaRecorder.stop();
    });
  }

  // Live elapsed time including any prior accumulated time before pause.
  elapsedMs(): number {
    if (this.state === "recording") {
      return this.accumulatedMs + (performance.now() - this.startTime);
    }
    return this.accumulatedMs;
  }
}
