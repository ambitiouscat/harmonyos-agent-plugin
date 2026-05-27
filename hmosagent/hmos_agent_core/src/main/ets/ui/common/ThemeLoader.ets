import { resourceManager } from '@kit.LocalizationKit';

export interface ThemeConfig {
  id: string;
  name: string;
  page_bg: string;
  card_bg: string;
  input_bg: string;
  text_primary: string;
  text_secondary: string;
  text_muted: string;
  accent: string;
  accent_danger: string;
  border: string;
  bubble_user_bg: string;
  bubble_ai_bg: string;
  code_bg: string;
  code_header_bg: string;
  code_text: string;
}

const DEFAULT_THEME: ThemeConfig = {
  id: 'dark',
  name: 'Dark',
  page_bg: '#1a1d23',
  card_bg: '#2a2d33',
  input_bg: '#2a2d35',
  text_primary: '#e0e0e0',
  text_secondary: '#d0d0d0',
  text_muted: '#666',
  accent: '#a0c4ff',
  accent_danger: '#e74c3c',
  border: 'rgba(255,255,255,0.08)',
  bubble_user_bg: '#1e3a5f',
  bubble_ai_bg: '#2a2d33',
  code_bg: '#1e2028',
  code_header_bg: '#2a2d35',
  code_text: '#c0c0c0'
};

export class ThemeLoader {
  private themes: ThemeConfig[] = [DEFAULT_THEME];
  private currentId: string = 'dark';
  private loaded: boolean = false;

  load(context: Context): void {
    if (this.loaded) return;
    try {
      const rm = context.resourceManager;
      const raw = rm.getRawFileContentSync('themes.json');
      const text = this.bytesToString(raw);
      const parsed = JSON.parse(text) as Array<Record<string, string>>;
      const list: ThemeConfig[] = [];
      for (let i = 0; i < parsed.length; i++) {
        const t = parsed[i];
        list.push({
          id: t['id'] as string || 'custom',
          name: t['name'] as string || 'Custom',
          page_bg: t['page_bg'] as string || '#fff',
          card_bg: t['card_bg'] as string || '#f5f5f5',
          input_bg: t['input_bg'] as string || '#f5f5f5',
          text_primary: t['text_primary'] as string || '#000',
          text_secondary: t['text_secondary'] as string || '#333',
          text_muted: t['text_muted'] as string || '#888',
          accent: t['accent'] as string || '#2196F3',
          accent_danger: t['accent_danger'] as string || '#e74c3c',
          border: t['border'] as string || 'rgba(0,0,0,0.08)',
          bubble_user_bg: t['bubble_user_bg'] as string || '#e3f2fd',
          bubble_ai_bg: t['bubble_ai_bg'] as string || '#f5f5f5',
          code_bg: t['code_bg'] as string || '#f0f0f0',
          code_header_bg: t['code_header_bg'] as string || '#e0e0e0',
          code_text: t['code_text'] as string || '#333',
        });
      }
      if (list.length > 0) this.themes = list;
    } catch (_e) {
      // fallback to default
    }
    this.loaded = true;
    // restore last choice
    try {
      const saved = AppStorage.get<string>('themeColor');
      if (saved && this.themes.find((t: ThemeConfig) => t.id === saved)) {
        this.currentId = saved;
      }
    } catch (_e) { /* ignore */ }
  }

  getCurrent(): ThemeConfig {
    const found = this.themes.find((t: ThemeConfig) => t.id === this.currentId);
    return found || DEFAULT_THEME;
  }

  getThemeNames(): string[] {
    return this.themes.map((t: ThemeConfig) => t.name);
  }

  getThemeIds(): string[] {
    return this.themes.map((t: ThemeConfig) => t.id);
  }

  getCurrentIndex(): number {
    return this.themes.findIndex((t: ThemeConfig) => t.id === this.currentId);
  }

  cycleNext(): void {
    const idx = this.getCurrentIndex();
    const next = (idx + 1) % this.themes.length;
    this.currentId = this.themes[next].id;
    AppStorage.setOrCreate('themeColor', this.currentId);
  }

  setTheme(id: string): void {
    if (this.themes.find((t: ThemeConfig) => t.id === id)) {
      this.currentId = id;
      AppStorage.setOrCreate('themeColor', this.currentId);
    }
  }

  private bytesToString(bytes: Uint8Array): string {
    let result = '';
    for (let i = 0; i < bytes.length; i++) {
      result += String.fromCharCode(bytes[i]);
    }
    return result;
  }
}

let instance: ThemeLoader | null = null;

export function getThemeLoader(): ThemeLoader {
  if (instance === null) {
    instance = new ThemeLoader();
  }
  return instance;
}
