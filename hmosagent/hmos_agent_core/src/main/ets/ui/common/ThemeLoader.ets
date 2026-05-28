import { preferences } from '@kit.ArkData';

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
  tool_colors: Record<string, string>;
}

const DEFAULT_THEME: ThemeConfig = {
  id: 'light',
  name: 'Light',
  page_bg: '#ffffff',
  card_bg: '#f5f5f5',
  input_bg: '#f5f5f5',
  text_primary: '#1a1a1a',
  text_secondary: '#333333',
  text_muted: '#888',
  accent: '#2196F3',
  accent_danger: '#e74c3c',
  border: 'rgba(0,0,0,0.08)',
  bubble_user_bg: '#e3f2fd',
  bubble_ai_bg: '#f5f5f5',
  code_bg: '#f0f0f0',
  code_header_bg: '#e0e0e0',
  code_text: '#333333',
  tool_colors: {}
};

export class ThemeLoader {
  private themes: ThemeConfig[] = [DEFAULT_THEME];
  private currentId: string = 'light';
  private loaded: boolean = false;
  private ctx: Context | null = null;

  load(context: Context): void {
    this.ctx = context;
    if (this.loaded) return;
    try {
      const rm = context.resourceManager;
      const raw = rm.getRawFileContentSync('themes.json');
      const text = this.bytesToString(raw);
      const parsed = JSON.parse(text) as Array<Record<string, Object>>;
      const list: ThemeConfig[] = [];
      for (let i = 0; i < parsed.length; i++) {
        const t = parsed[i];
        list.push({
          id: (t['id'] as string) || 'custom',
          name: (t['name'] as string) || 'Custom',
          page_bg: (t['page_bg'] as string) || '#fff',
          card_bg: (t['card_bg'] as string) || '#f5f5f5',
          input_bg: (t['input_bg'] as string) || '#f5f5f5',
          text_primary: (t['text_primary'] as string) || '#000',
          text_secondary: (t['text_secondary'] as string) || '#333',
          text_muted: (t['text_muted'] as string) || '#888',
          accent: (t['accent'] as string) || '#2196F3',
          accent_danger: (t['accent_danger'] as string) || '#e74c3c',
          border: (t['border'] as string) || 'rgba(0,0,0,0.08)',
          bubble_user_bg: (t['bubble_user_bg'] as string) || '#e3f2fd',
          bubble_ai_bg: (t['bubble_ai_bg'] as string) || '#f5f5f5',
          code_bg: (t['code_bg'] as string) || '#f0f0f0',
          code_header_bg: (t['code_header_bg'] as string) || '#e0e0e0',
          code_text: (t['code_text'] as string) || '#333',
          tool_colors: (t['tool_colors'] as Record<string, string>) || {},
        });
      }
      if (list.length > 0) this.themes = list;
    } catch (_e) {
      // fallback to default
    }
    this.loaded = true;

    // Fast path: AppStorage (same-session memory)
    try {
      const saved = AppStorage.get<string>('themeColor');
      if (saved && this.themes.find((t: ThemeConfig) => t.id === saved)) {
        this.currentId = saved;
        return;
      }
    } catch (_e) { /* ignore */ }

    // Slow path: restore from persistent preferences (cold start)
    this.restoreFromPrefs();
  }

  private async restoreFromPrefs(): Promise<void> {
    if (!this.ctx) return;
    try {
      const prefs = await preferences.getPreferences(this.ctx, 'hmos_agent_config');
      const saved = await prefs.get('theme_color', 'light') as string;
      if (saved && this.themes.find((t: ThemeConfig) => t.id === saved)) {
        this.currentId = saved;
        AppStorage.setOrCreate('themeColor', this.currentId);
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
    this.persistTheme();
  }

  setTheme(id: string): void {
    if (this.themes.find((t: ThemeConfig) => t.id === id)) {
      this.currentId = id;
      AppStorage.setOrCreate('themeColor', this.currentId);
      this.persistTheme();
    }
  }

  private persistTheme(): void {
    if (!this.ctx) return;
    preferences.getPreferences(this.ctx, 'hmos_agent_config')
      .then((prefs: preferences.Preferences) => {
        prefs.put('theme_color', this.currentId);
        prefs.flush();
      })
      .catch(() => { /* ignore */ });
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
