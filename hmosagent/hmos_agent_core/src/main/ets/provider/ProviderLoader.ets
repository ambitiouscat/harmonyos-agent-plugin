import { http } from '@kit.NetworkKit';
import { fileIo as fs } from '@kit.CoreFileKit';

export interface OpenCodeModel {
  id: string;
  name?: string;
}
export interface OpenCodeProvider {
  id: string;
  name: string;
  api?: string;
  npm?: string;
  models?: Record<string, OpenCodeModel>;
}

const REMOTE_URL = 'https://models.dev/api/json';
const CACHE_FILE = 'providers.json';

export class ProviderLoader {
  private providers: Record<string, OpenCodeProvider> = {};
  private sortedKeys: string[] = [];
  private onUpdate?: () => void;

  setUpdateCallback(cb: () => void): void {
    this.onUpdate = cb;
  }

  getProviders(): Record<string, OpenCodeProvider> {
    return this.providers;
  }

  getSortedKeys(): string[] {
    return this.sortedKeys;
  }

  /**
   * Load providers: embedded fallback first, then try cache, then remote.
   */
  async load(context: Context): Promise<void> {
    // 1. Load embedded fallback immediately (non-blocking UI)
    await this.loadEmbedded(context);
    this.sortKeys();

    // 2. Try sandbox cache
    const cached = await this.loadCache(context);
    if (cached) {
      this.providers = cached;
      this.sortKeys();
      this.notifyUpdate();
    }

    // 3. Fetch remote in background
    try {
      const remote = await this.fetchRemote();
      if (remote) {
        this.providers = remote;
        this.sortKeys();
        await this.saveCache(context, remote);
        this.notifyUpdate();
      }
    } catch (_e) {
      // Remote failed — keep using cache/embedded
    }
  }

  private sortKeys(): void {
    // Put common providers at top
    const top: string[] = ['openai', 'anthropic', 'deepseek', 'google', 'groq', 'mistral'];
    const all = Object.keys(this.providers).sort();
    this.sortedKeys = [...top.filter(k => all.includes(k)), ...all.filter(k => !top.includes(k))];
  }

  private notifyUpdate(): void {
    if (this.onUpdate) this.onUpdate();
  }

  // Decode UTF-8 bytes to string without spread operator (avoids argument count limit)
  private utf8BytesToString(bytes: Uint8Array): string {
    let result = '';
    let i = 0;
    while (i < bytes.length) {
      const byte = bytes[i];
      let codePoint: number;
      let advance: number;
      if (byte < 0x80) {
        codePoint = byte;
        advance = 1;
      } else if ((byte & 0xE0) === 0xC0) {
        codePoint = ((byte & 0x1F) << 6) | (bytes[i + 1] & 0x3F);
        advance = 2;
      } else if ((byte & 0xF0) === 0xE0) {
        codePoint = ((byte & 0x0F) << 12) | ((bytes[i + 1] & 0x3F) << 6) | (bytes[i + 2] & 0x3F);
        advance = 3;
      } else if ((byte & 0xF8) === 0xF0) {
        codePoint = ((byte & 0x07) << 18) | ((bytes[i + 1] & 0x3F) << 12) |
          ((bytes[i + 2] & 0x3F) << 6) | (bytes[i + 3] & 0x3F);
        advance = 4;
      } else {
        i++;
        continue;
      }
      result += String.fromCodePoint(codePoint);
      i += advance;
    }
    return result;
  }

  // ── embedded fallback from rawfile ──
  private async loadEmbedded(context: Context): Promise<void> {
    try {
      const raw = await context.resourceManager.getRawFileContent(CACHE_FILE);
      const bytes = new Uint8Array(raw.buffer);
      const text = this.utf8BytesToString(bytes);
      this.providers = JSON.parse(text) as Record<string, OpenCodeProvider>;
    } catch (e) {
      console.error('[ProviderLoader] loadEmbedded failed: ' + e);
      this.providers = {};
      this.providers['openai'] = { id: 'openai', name: 'OpenAI', api: 'https://api.openai.com/v1' };
      this.providers['anthropic'] = { id: 'anthropic', name: 'Anthropic', api: 'https://api.anthropic.com/v1' };
      this.providers['deepseek'] = { id: 'deepseek', name: 'DeepSeek', api: 'https://api.deepseek.com' };
    }
  }

  // ── sandbox cache ──
  private async loadCache(context: Context): Promise<Record<string, OpenCodeProvider> | null> {
    try {
      const path = context.filesDir + '/' + CACHE_FILE;
      const stat = fs.statSync(path);
      // Only use cache if not older than 7 days
      const ageHours = (Date.now() - stat.mtime * 1000) / 3600000;
      if (ageHours > 168) return null; // 7 days
      const content = fs.readTextSync(path);
      return JSON.parse(content) as Record<string, OpenCodeProvider>;
    } catch (_e) {
      return null;
    }
  }

  private async saveCache(context: Context, data: Record<string, OpenCodeProvider>): Promise<void> {
    try {
      const path = context.filesDir + '/' + CACHE_FILE;
      const file = fs.openSync(path, fs.OpenMode.CREATE | fs.OpenMode.WRITE_ONLY);
      fs.writeSync(file.fd, JSON.stringify(data));
      fs.closeSync(file);
    } catch (_e) {
      // silent
    }
  }

  // ── remote fetch ──
  private async fetchRemote(): Promise<Record<string, OpenCodeProvider> | null> {
    return new Promise<Record<string, OpenCodeProvider> | null>((resolve) => {
      try {
        const req = http.createHttp();
        req.request(REMOTE_URL, { method: http.RequestMethod.GET, connectTimeout: 10000, readTimeout: 30000 },
          (err, resp) => {
            if (err || !resp) {
              resolve(null);
              return;
            }
            try {
              const body = typeof resp.result === 'string' ? resp.result : JSON.stringify(resp.result);
              const data = JSON.parse(body) as Record<string, OpenCodeProvider>;
              resolve(data);
            } catch (_e) {
              resolve(null);
            }
          });
      } catch (_e) {
        resolve(null);
      }
    });
  }
}

// Singleton for app-wide use
const instance = new ProviderLoader();
export function getProviderLoader(): ProviderLoader {
  return instance;
}
