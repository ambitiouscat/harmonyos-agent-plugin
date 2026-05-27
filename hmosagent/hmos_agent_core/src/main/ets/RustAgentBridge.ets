import nativeBridge from 'libnative_bridge.so';

type StreamCallback = (data: string, eventType: number) => void;
type IoCallback = (url: string, body: string) => string;
type OfflineCallback = (isOffline: boolean) => void;

export class RustAgentBridge {
  private static instance: RustAgentBridge | null = null;

  private streamCallback: StreamCallback | null = null;
  private ioCallback: IoCallback | null = null;
  private offlineCallback: OfflineCallback | null = null;
  private initialized: boolean = false;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 3;
  private isOffline: boolean = false;

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
   * Initialize the session manager with the host's sandbox files directory.
   * Must be called once after bridge init to enable session CRUD.
   */
  public initSession(filesDir: string): string {
    return this.call('init_session', JSON.stringify({ files_dir: filesDir }));
  }

  /**
   * Register a callback for offline/online state changes.
   */
  public onOfflineChange(cb: OfflineCallback): void {
    this.offlineCallback = cb;
  }

  /**
   * Retry an action with exponential backoff on failure.
   * Returns the result if successful, or the last error after max retries.
   */
  public async callWithRetry(action: string, args: string): Promise<string> {
    let lastError = '';
    for (let attempt = 0; attempt <= this.maxReconnectAttempts; attempt++) {
      try {
        const result = this.call(action, args);
        const parsed = JSON.parse(result) as Record<string, string>;
        if (parsed['status'] === 'error') {
          lastError = parsed['error'] || 'Unknown error';
        } else {
          this.reconnectAttempts = 0;
          if (this.isOffline) {
            this.isOffline = false;
            if (this.offlineCallback) this.offlineCallback(false);
          }
          return result;
        }
      } catch (e) {
        lastError = `${e}`;
      }

      if (attempt < this.maxReconnectAttempts) {
        this.reconnectAttempts = attempt + 1;
        const delayMs = Math.min(500 * Math.pow(2, attempt), 5000);
        console.info(`[Bridge] Retry ${attempt + 1}/${this.maxReconnectAttempts} in ${delayMs}ms`);
        await this.sleep(delayMs);
      }
    }

    this.reconnectAttempts = this.maxReconnectAttempts;
    if (!this.isOffline) {
      this.isOffline = true;
      if (this.offlineCallback) this.offlineCallback(true);
    }
    return `{"status":"error","error":"All retries exhausted: ${lastError}"}`;
  }

  /**
   * Check if the bridge is currently in offline mode.
   */
  public getIsOffline(): boolean {
    return this.isOffline;
  }

  /**
   * Reset offline state (called when user manually retries from UI).
   */
  public resetOffline(): void {
    this.isOffline = false;
    this.reconnectAttempts = 0;
    if (this.offlineCallback) this.offlineCallback(false);
  }

  private sleep(ms: number): Promise<void> {
    return new Promise<void>((resolve) => {
      setTimeout(() => resolve(), ms);
    });
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
