import { util } from '@kit.ArkTS';

interface PhraseWordGroup {
  name: string;
  words: string[];
}

interface PhrasePreset {
  name: string;
  lang: string;
  words: string[] | Record<string, PhraseWordGroup>;
  flat?: string[];
}

interface CompletionWords {
  name: string;
  words: Record<string, string[]>;
}

interface PhraseData {
  presets: Record<string, PhrasePreset>;
  completions: CompletionWords;
}

const SPINNER_TO_COMPLETION: Record<string, string> = {
  'claude-original': 'claude-original',
  'japanese': 'claude-original',
  'chinese-history-en': 'claude-original',
  'bilingual-mix': 'chinese-cooking',
  'classical-chinese': 'chinese-history',
  'wuxia': 'chinese-history',
  'chinese-history': 'chinese-history',
  'programmer-humor': 'chinese-programmer',
  'cyberpunk': 'cyberpunk-completion',
  'cute': 'cute-completion',
  'simple-chinese': 'simple-chinese-completion'
};

const FALLBACK_WORDS: string[] = [
  'Thinking', 'Processing', 'Computing', 'Brewing', 'Baking',
  'Cooking', 'Forging', 'Synthesizing', 'Calculating', 'Pondering'
];

const FALLBACK_COMPLETIONS: string[] = ['Done!', 'Baked!', 'Worked!', 'Brewed!'];

export class PhraseLoader {
  private static instance: PhraseLoader;
  private data: PhraseData | null = null;
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
      this.loaded = true;
    } catch (e) {
      console.error('[PhraseLoader] load failed, using fallback: ' + e);
      this.loaded = true;
    }
  }

  getSpinnerPhrases(styleKey: string): string[] {
    if (!this.data || !this.data.presets) {
      return FALLBACK_WORDS;
    }
    const preset = this.data.presets[styleKey];
    if (!preset) {
      return this.data.presets['claude-original']?.words as string[] || FALLBACK_WORDS;
    }
    if (styleKey === 'chinese-history') {
      if (Array.isArray(preset.flat)) {
        return preset.flat;
      }
      const words = preset.words;
      if (Array.isArray(words)) {
        return words;
      }
      return FALLBACK_WORDS;
    }
    if (Array.isArray(preset.words)) {
      return preset.words;
    }
    return FALLBACK_WORDS;
  }

  getRandomCompletion(styleKey: string): string {
    const compKey = SPINNER_TO_COMPLETION[styleKey];
    if (!compKey) {
      return this.pickRandom(FALLBACK_COMPLETIONS);
    }
    if (this.data && this.data.completions && this.data.completions.words) {
      const compWords = this.data.completions.words[compKey];
      if (compWords && compWords.length > 0) {
        return this.pickRandom(compWords);
      }
    }
    const fallbackGroup = this.data?.completions?.words?.['claude-original'];
    if (fallbackGroup && fallbackGroup.length > 0) {
      return this.pickRandom(fallbackGroup);
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
