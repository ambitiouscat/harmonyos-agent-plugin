import { preferences } from '@kit.ArkData';
import { getProviderLoader, OpenCodeModel, OpenCodeProvider } from '../provider/ProviderLoader';
import { getPhraseLoader, CategoryInfo } from '../ui/common/PhraseLoader';

@ComponentV2
export struct SettingsPage {
  @Local apiKey: string = '';
  @Local baseUrl: string = '';
  @Local model: string = '';
  @Local providerId: string = 'openai';
  @Local saved: boolean = false;
  @Local showProviderPicker: boolean = false;
  @Local showModelPicker: boolean = false;
  @Local showPhraseStylePicker: boolean = false;
  @Local providerSearch: string = '';
  @Local thinkingPhraseStyle: string = 'claude-original';
  @Local phraseCategories: CategoryInfo[] = [];
  @Param onSettingsSaved: () => void = () => {};

  private prefs: preferences.Preferences | null = null;
  private loader = getProviderLoader();

  aboutToAppear(): void {
    this.loader.setUpdateCallback(() => { this.onProvidersUpdated(); });
    this.loader.load(getContext(this));
    this.loadConfig();
    this.loadPhraseCategories();
  }

  async loadPhraseCategories(): Promise<void> {
    await getPhraseLoader().load(getContext(this));
    const cats = getPhraseLoader().getCategories();
    if (cats.length > 0) {
      this.phraseCategories = cats;
    } else {
      this.phraseCategories = [{
        id: 'claude-original',
        name: 'Claude Code Original (187)',
        phrases: [],
        completions: []
      }];
    }
  }

  @Local providerVersion: number = 0;

  onProvidersUpdated(): void {
    this.providerVersion++;
  }

  get providers(): Record<string, OpenCodeProvider> {
    return this.loader.getProviders();
  }

  get providerList(): string[] {
    return this.loader.getSortedKeys();
  }

  get filteredProviderList(): string[] {
    if (!this.providerSearch) return this.providerList;
    const q = this.providerSearch.toLowerCase();
    return this.providerList.filter((id: string) => {
      const p = this.getProvider(id);
      if (!p) return false;
      return p.name.toLowerCase().includes(q) || id.toLowerCase().includes(q);
    });
  }

  getProvider(id: string): OpenCodeProvider | undefined {
    return this.providers[id];
  }

  getProviderUrl(id: string): string {
    const p = this.getProvider(id);
    if (!p || !p.api) return '';
    // OpenCode's "api" already ends with /v1 for most providers.
    // Some (like deepseek) don't — ensure /v1 suffix.
    if (p.api.endsWith('/v1') || p.api.includes('/chat/completions')) return p.api;
    return p.api + '/v1';
  }

  getProviderModels(): OpenCodeModel[] {
    const p = this.getProvider(this.providerId);
    if (!p || !p.models) return [];
    return Object.values(p.models);
  }

  selectProvider(id: string): void {
    this.providerId = id;
    this.showProviderPicker = false;
    this.providerSearch = '';
    this.baseUrl = this.getProviderUrl(id);
    const models = this.getProviderModels();
    if (models.length > 0) this.model = models[0].id;
  }

  getSelectedProviderName(): string {
    const p = this.getProvider(this.providerId);
    return p ? p.name : 'Custom';
  }

  getCurrentPhraseStyleName(): string {
    for (const cat of this.phraseCategories) {
      if (cat.id === this.thinkingPhraseStyle) {
        return cat.name;
      }
    }
    return 'Select style...';
  }

  async loadConfig(): Promise<void> {
    try {
      this.prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
      this.apiKey = await this.prefs.get('api_key', '') as string;
      this.baseUrl = await this.prefs.get('base_url', '') as string;
      this.model = await this.prefs.get('model', '') as string;
      this.providerId = await this.prefs.get('provider', 'openai') as string;
      this.thinkingPhraseStyle = await this.prefs.get('thinking_phrase_style', 'claude-original') as string;
      if (!this.baseUrl) this.baseUrl = this.getProviderUrl(this.providerId);
      if (!this.model) {
        const models = this.getProviderModels();
        if (models.length > 0) this.model = models[0].id;
      }
    } catch (e) {
      console.error('[SettingsPage] loadConfig failed: ' + e);
    }
  }

  async saveConfig(): Promise<void> {
    if (!this.prefs) return;
    if (!this.apiKey || !this.baseUrl) return;
    try {
      await this.prefs.put('api_key', this.apiKey);
      await this.prefs.put('base_url', this.baseUrl);
      await this.prefs.put('model', this.model);
      await this.prefs.put('provider', this.providerId);
      await this.prefs.put('thinking_phrase_style', this.thinkingPhraseStyle);
      AppStorage.setOrCreate('thinking_phrase_style', this.thinkingPhraseStyle);
      await this.prefs.flush();

      this.saved = true;
      setTimeout(() => { this.saved = false; }, 2000);
      if (this.onSettingsSaved) this.onSettingsSaved();
    } catch (e) {
      console.error('[SettingsPage] saveConfig failed: ' + e);
    }
  }

  build() {
    Column({ space: 12 }) {
      Text('AI Settings')
        .fontSize(18)
        .fontWeight(FontWeight.Bold)
        .margin({ top: 8 })

      // Provider picker
      Column() {
        Text('Provider').fontSize(12).fontColor('#999')
        Row() {
          Text(this.getSelectedProviderName()).fontSize(14).layoutWeight(1)
          Button(this.showProviderPicker ? '▲' : '▼')
            .fontSize(12).width(28).height(28)
            .backgroundColor('#f0f0f0')
            .hitTestBehavior(HitTestMode.None)
        }
        .width('100%').padding(8).border({ width: 1, color: '#ddd' }).borderRadius(4)
        .onClick(() => {
          this.showProviderPicker = !this.showProviderPicker;
          if (this.showProviderPicker) this.providerSearch = '';
        })

        if (this.showProviderPicker) {
          Column() {
            TextInput({ text: $$this.providerSearch, placeholder: 'Search provider...' })
              .fontSize(13).height(36)
              .padding({ left: 8, right: 8 })
              .margin({ left: 4, right: 4, top: 4, bottom: 4 })
            List({ space: 0 }) {
              ForEach(this.filteredProviderList, (id: string) => {
                ListItem() {
                  Row() {
                    Text(this.getProvider(id)?.name || id).fontSize(13).fontColor('#333')
                    Blank()
                    if (id === this.providerId) Text('✓').fontSize(13).fontColor('#4CAF50')
                  }
                  .width('100%').padding({ left: 12, right: 12, top: 8, bottom: 8 })
                  .onClick(() => { this.selectProvider(id); })
                }
              }, (id: string) => id)
            }
            .constraintSize({ maxHeight: 240 })
            .backgroundColor('#fff')
            .border({ width: 1, color: '#ddd' }).borderRadius(4)
          }
          .margin({ top: 2 })
        }
      }.width('100%')

      // API Key
      Column() {
        Text('API Key').fontSize(12).fontColor('#999')
        TextInput({ text: $$this.apiKey, placeholder: 'sk-...' })
          .type(InputType.Password).fontSize(14)
      }.width('100%')

      // Model selection
      Column() {
        Text('Model').fontSize(12).fontColor('#999')
        Row() {
          Text(this.model || 'Select a model')
            .fontSize(14)
            .layoutWeight(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .maxLines(1)
          Button(this.showModelPicker ? '▲' : '▼')
            .fontSize(12).width(28).height(28)
            .backgroundColor('#f0f0f0')
            .hitTestBehavior(HitTestMode.None)
        }
        .width('100%').padding(8)
        .border({ width: 1, color: '#ddd' }).borderRadius(4)
        .onClick(() => { this.showModelPicker = !this.showModelPicker; })

        if (this.showModelPicker) {
          Column() {
            if (this.getProviderModels().length > 0) {
              List({ space: 0 }) {
                ForEach(this.getProviderModels(), (m: OpenCodeModel) => {
                  ListItem() {
                    Row() {
                      Text(m.name || m.id)
                        .fontSize(12)
                        .fontColor(this.model === m.id ? '#2196F3' : '#333')
                        .layoutWeight(1)
                        .textOverflow({ overflow: TextOverflow.Ellipsis })
                        .maxLines(1)
                      if (m.id !== (m.name || m.id)) {
                        Text(m.id)
                          .fontSize(10).fontColor('#999')
                          .margin({ left: 6 })
                      }
                      Blank()
                      if (this.model === m.id) {
                        Text('✓').fontSize(13).fontColor('#4CAF50')
                      }
                    }
                    .width('100%').padding({ left: 12, right: 12, top: 6, bottom: 6 })
                    .onClick(() => {
                      this.model = m.id;
                      this.showModelPicker = false;
                    })
                  }
                }, (m: OpenCodeModel) => m.id)
              }
              .constraintSize({ maxHeight: 240 })
              .backgroundColor('#fff')
              .border({ width: 1, color: '#ddd' }).borderRadius(4)
            } else {
              Text('No models available')
                .fontSize(12).fontColor('#999').padding(12)
            }
          }
          .margin({ top: 2 })
        }

        TextInput({ text: $$this.model, placeholder: 'Or enter model ID manually' })
          .fontSize(13).margin({ top: 6 })
      }.width('100%')

      // Thinking phrase style picker
      Column() {
        Text('Thinking Phrase Style').fontSize(12).fontColor('#999')
        Row() {
          Text(this.getCurrentPhraseStyleName())
            .fontSize(13)
            .layoutWeight(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .maxLines(1)
          Button(this.showPhraseStylePicker ? '▲' : '▼')
            .fontSize(12).width(28).height(28)
            .backgroundColor('#f0f0f0')
            .hitTestBehavior(HitTestMode.None)
        }
        .width('100%').padding(8)
        .border({ width: 1, color: '#ddd' }).borderRadius(4)
        .onClick(() => { this.showPhraseStylePicker = !this.showPhraseStylePicker; })

        if (this.showPhraseStylePicker) {
          List({ space: 0 }) {
            ForEach(this.phraseCategories, (cat: CategoryInfo) => {
              ListItem() {
                Row() {
                  Text(cat.name)
                    .fontSize(12)
                    .fontColor(this.thinkingPhraseStyle === cat.id ? '#2196F3' : '#333')
                    .layoutWeight(1)
                  if (this.thinkingPhraseStyle === cat.id) {
                    Text('✓').fontSize(13).fontColor('#4CAF50')
                  }
                }
                .width('100%').padding({ left: 12, right: 12, top: 6, bottom: 6 })
                .onClick(() => {
                  this.thinkingPhraseStyle = cat.id;
                  this.showPhraseStylePicker = false;
                })
              }
            }, (cat: CategoryInfo) => cat.id)
          }
          .constraintSize({ maxHeight: 240 })
          .backgroundColor('#fff')
          .border({ width: 1, color: '#ddd' }).borderRadius(4)
          .margin({ top: 2 })
        }
      }.width('100%')

      // Base URL
      Column() {
        Text('Base URL').fontSize(12).fontColor('#999')
        TextInput({ text: $$this.baseUrl, placeholder: 'https://api.openai.com/v1' })
          .fontSize(13)
      }.width('100%')

      Button(this.saved ? '✓ Saved' : 'Save Configuration')
        .fontSize(14)
        .onClick(() => this.saveConfig())
    }
    .width('100%')
    .padding(14)
  }
}
