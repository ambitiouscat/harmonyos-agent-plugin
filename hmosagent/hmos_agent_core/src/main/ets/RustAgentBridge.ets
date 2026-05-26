import nativeBridge from 'libnative_bridge.so';

type StreamCallback = (data: string, eventType: number) => void;
type IoCallback = (url: string, body: string) => string;

export class RustAgentBridge {
  private static instance: RustAgentBridge | null = null;

  private streamCallback: StreamCallback | null = null;
  private ioCallback: IoCallback | null = null;
  private initialized: boolean = false;

  static getInstance(): RustAgentBridge {
    if (RustAgentBridge.instance === null) {
      RustAgentBridge.instance = new RustAgentBridge();
    }
    return RustAgentBridge.instance;
  }

  /**
   * Initialize the Rust agent core with stream-only callback (Phase 0 compat).
   * eventType: 0 = data chunk, 1 = stream completed, 2 = error
   */
  public init(onChunk: StreamCallback): boolean {
    if (this.initialized) {
      this.streamCallback = onChunk;
      return true;
    }
    this.streamCallback = onChunk;
    try {
      this.initialized = nativeBridge.initAgent((data: string, eventType: number) => {
        if (this.streamCallback) {
          this.streamCallback(data, eventType);
        }
      });
      return this.initialized;
    } catch (e) {
      console.error(`RustAgentBridge init failed: ${e}`);
      return false;
    }
  }

  /**
   * Initialize with full IO injection (Phase 1).
   * @param onChunk  Streaming delta callback (eventType 0/1/2).
   * @param onIo     Blocking IO callback invoked by Rust's post_fn.
   *                 Receives (url, body), MUST return response string.
   */
  public initWithIo(onChunk: StreamCallback, onIo: IoCallback): boolean {
    if (this.initialized) {
      this.streamCallback = onChunk;
      this.ioCallback = onIo;
      return true;
    }
    this.streamCallback = onChunk;
    this.ioCallback = onIo;
    try {
      this.initialized = nativeBridge.initAgentWithIo(
        (data: string, eventType: number) => {
          if (this.streamCallback) {
            this.streamCallback(data, eventType);
          }
        },
        (url: string, body: string): string => {
          if (this.ioCallback) {
            return this.ioCallback(url, body);
          }
          return '{"status":"error","error":"IO callback not set"}';
        }
      );
      return this.initialized;
    } catch (e) {
      console.error(`RustAgentBridge initWithIo failed: ${e}`);
      return false;
    }
  }

  /**
   * Call a Rust action with JSON arguments.
   */
  public call(action: string, jsonArgs: string): string {
    if (!this.initialized) {
      return '{"status":"error","error":"Agent not initialized"}';
    }
    return nativeBridge.agentCall(action, jsonArgs);
  }

  /**
   * Test network connectivity via Rust sandbox.
   */
  public testNetwork(): boolean {
    return nativeBridge.testNetwork();
  }

  /**
   * Search files in a directory (in-process ripgrep).
   * Returns JSON array of matches.
   */
  public search(dirPath: string, pattern: string): string {
    return nativeBridge.search(dirPath, pattern);
  }

  /**
   * Scan a directory into the RAG index.
   * Returns JSON with {status, chunks_indexed}.
   */
  public scanDir(dirPath: string): string {
    return nativeBridge.scanDir(dirPath);
  }

  /**
   * Abort the current generation. Sets the ABORT_FLAG in Rust core,
   * causing the ReAct loop to stop at the next iteration boundary.
   */
  public abort(): string {
    return this.call("abort", "{}");
  }

  /**
   * Reset the abort flag before starting a new generation.
   * Must be called by the TS side before sending a new prompt
   * to clear any stale abort signal.
   */
  public resetAbort(): string {
    return this.call("reset_abort", "{}");
  }
}
