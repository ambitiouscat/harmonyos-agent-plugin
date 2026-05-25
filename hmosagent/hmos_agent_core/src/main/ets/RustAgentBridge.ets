import nativeBridge from 'libnative_bridge.so';

type StreamCallback = (data: string, eventType: number) => void;

export class RustAgentBridge {
  private streamCallback: StreamCallback | null = null;
  private initialized: boolean = false;

  /**
   * Initialize the Rust agent core with a streaming callback.
   * eventType: 0 = data chunk, 1 = stream completed, 2 = error
   */
  public init(onChunk: StreamCallback): boolean {
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
}
