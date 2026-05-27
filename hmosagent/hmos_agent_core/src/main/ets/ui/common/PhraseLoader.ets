import { util } from '@kit.ArkTS';

export interface CategoryInfo {
  id: string;
  name: string;
  lang?: string;
  phrases: string[];
  completions: string[];
}

interface PhraseData {
  categories: CategoryInfo[];
}

const FALLBACK_WORDS: string[] = [
  'Thinking', 'Processing', 'Computing', 'Brewing', 'Baking',
  'Cooking', 'Forging', 'Synthesizing', 'Calculating', 'Pondering'
];

const FALLBACK_COMPLETIONS: string[] = ['Done!', 'Baked!', 'Worked!', 'Brewed!'];

export class PhraseLoader {
  private static instance: PhraseLoader;
  private data: PhraseData | null = null;
  private categoryMap: Map<string, CategoryInfo> = new Map();
  private loaded: boolean = false;

  static getInstance(): PhraseLoader {
    if (!PhraseLoader.instance) {
      PhraseLoader.instance = new PhraseLoader();
    }
    return PhraseLoader.instance;
  }

  async load(context: Context): Promise<void> {
    if (this.loaded) return;
    try {
      const raw = await context.resourceManager.getRawFileContent('agent_thinking_phrases.json');
      const decoder = new util.TextDecoder('utf-8');
      const jsonStr = decoder.decodeToString(raw);
      this.data = JSON.parse(jsonStr) as PhraseData;
      this.categoryMap.clear();
      for (const cat of this.data.categories) {
        this.categoryMap.set(cat.id, cat);
      }
      this.loaded = true;
    } catch (e) {
      console.error('[PhraseLoader] load failed, using fallback: ' + e);
      this.loaded = true;
    }
  }

  getCategories(): CategoryInfo[] {
    if (!this.data) return [];
    return this.data.categories;
  }

  getSpinnerPhrases(styleKey: string): string[] {
    const cat = this.categoryMap.get(styleKey);
    if (!cat || cat.phrases.length === 0) {
      return this.categoryMap.get('claude-original')?.phrases || FALLBACK_WORDS;
    }
    return cat.phrases;
  }

  getRandomCompletion(styleKey: string): string {
    const cat = this.categoryMap.get(styleKey);
    if (cat && cat.completions.length > 0) {
      return this.pickRandom(cat.completions);
    }
    const def = this.categoryMap.get('claude-original');
    if (def && def.completions.length > 0) {
      return this.pickRandom(def.completions);
    }
    return this.pickRandom(FALLBACK_COMPLETIONS);
  }

  private pickRandom(arr: string[]): string {
    return arr[Math.floor(Math.random() * arr.length)];
  }
}

export function getPhraseLoader(): PhraseLoader {
  return PhraseLoader.getInstance();
}
