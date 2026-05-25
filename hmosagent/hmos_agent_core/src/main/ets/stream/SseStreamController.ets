/**
 * SSE Stream Controller — decoupled streaming engine.
 *
 * Parses SSE (Server-Sent Events) response body and emits text deltas
 * with inter-chunk delays to give the ArkUI render loop time to paint,
 * producing a visual streaming effect.
 *
 * Usage:
 *   const ctrl = new SseStreamController();
 *   ctrl.process(body, {
 *     onDelta: (text) => { ... },
 *     onDone:  ()    => { ... },
 *     onError: (msg) => { ... },
 *   });
 *   // Later: ctrl.cancel() to abort mid-stream.
 */

export interface SseCallbacks {
  onDelta: (text: string) => void;
  onDone: () => void;
  onError: (msg: string) => void;
}

export class SseStreamController {
  private timerIds: number[] = [];
  private cancelled: boolean = false;
  readonly delayMs: number;

  constructor(delayMs: number = 20) {
    this.delayMs = delayMs;
  }

  /** Parse `body` as SSE and emit deltas with staggered timers. */
  process(body: string, callbacks: SseCallbacks): void {
    this.cancelled = false;
    this.timerIds = [];

    // If response looks like JSON (non-streaming error), handle directly
    if (body.startsWith('{')) {
      try {
        const json = JSON.parse(body) as Record<string, Object>;
        if (json["error"] !== undefined) {
          const err = json["error"] as Record<string, Object>;
          callbacks.onError(err["message"] as string || JSON.stringify(json["error"]));
          return;
        }
      } catch (_e) { /* fall through to SSE parsing */ }
    }

    const events = body.split('\n\n');
    let chunkIndex = 0;
    let hasContent = false;

    for (const event of events) {
      if (!event || event.trim() === '') continue;
      const lines = event.split('\n');
      for (const line of lines) {
        if (!line.startsWith('data: ')) continue;
        const data = line.substring(6).trim();
        if (data === '[DONE]') {
          if (!hasContent) this.schedule(() => callbacks.onDone(), chunkIndex++);
          return;
        }
        try {
          const chunk = JSON.parse(data) as Record<string, Object>;
          const choices = chunk["choices"] as Array<Record<string, Object>>;
          if (choices && choices.length > 0) {
            const delta = choices[0]["delta"] as Record<string, Object>;
            const content = delta ? delta["content"] as string : undefined;
            if (content) {
              hasContent = true;
              this.schedule(() => callbacks.onDelta(content), chunkIndex++);
            }
          }
        } catch (_e) { /* skip malformed SSE chunks */ }
      }
    }

    // Stream ended without [DONE] marker
    this.schedule(() => {
      if (!hasContent && !this.cancelled) {
        callbacks.onError('(no content in response)');
      } else {
        callbacks.onDone();
      }
    }, chunkIndex++);
  }

  /** Cancel all pending scheduled callbacks. */
  cancel(): void {
    this.cancelled = true;
    for (const id of this.timerIds) {
      clearTimeout(id);
    }
    this.timerIds = [];
  }

  private schedule(fn: () => void, index: number): void {
    if (this.cancelled) return;
    const delay = index * this.delayMs;
    // For index 0, use microtask via setTimeout(0) for immediate-ish delivery
    const id = setTimeout(() => {
      if (!this.cancelled) fn();
    }, Math.max(0, delay));
    this.timerIds.push(id);
  }
}
