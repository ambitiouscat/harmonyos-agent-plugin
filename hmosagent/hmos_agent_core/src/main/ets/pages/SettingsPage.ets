import { preferences } from '@kit.ArkData';
import { fileIo, picker } from '@kit.CoreFileKit';
import { getProviderLoader, OpenCodeModel, OpenCodeProvider } from '../provider/ProviderLoader';
import { getPhraseLoader, CategoryInfo } from '../ui/common/PhraseLoader';
import { RustAgentBridge } from '../RustAgentBridge';

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
  @Local importStatus: string = '';
  @Local importDialogVisible: boolean = false;
  @Local importUri: string = '';
  @Local workspaceMode: string = '';
  @Local skillName: string = '';
  @Local skillDesc: string = '';
  @Local skillBody: string = '';

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

  async pickAndImportFolder(): Promise<void> {
    if (this.skillName && this.skillBody) {
      // Quick create via text input
      await this.createSkillFromText();
      return;
    }
    // Pick a file from device storage
    try {
      const options = new picker.DocumentSelectOptions();
      options.maxSelectNumber = 1;
      const docPicker = new picker.DocumentViewPicker(getContext(this));
      const uris = await docPicker.select(options);
      if (uris && uris.length > 0) {
        const uri = uris[0];
        const lastSlash = uri.lastIndexOf('/');
        const parentUri = lastSlash > 0 ? uri.substring(0, lastSlash) : uri;
        const folderName = parentUri.substring(parentUri.lastIndexOf('/') + 1);
        await this.importFromUri(parentUri, folderName);
      }
    } catch (e) {
      console.error('[Settings] pickAndImport failed: ' + JSON.stringify(e));
      this.importStatus = 'Import failed: ' + (typeof e === 'object' ? JSON.stringify(e) : '');
    }
  }

  async importFromUri(srcUri: string, folderName: string): Promise<void> {
    this.importStatus = 'Scanning...';
    try {
      const files = fileIo.listFileSync(srcUri);
      let hasSkillMd = false;
      let hasPluginJson = false;
      for (const f of files) {
        if (f.endsWith('SKILL.md') || f === 'SKILL.md') {
          hasSkillMd = true;
        }
        if (f.endsWith('plugin.json') || f === 'plugin.json') {
          hasPluginJson = true;
        }
      }
      if (!hasSkillMd && !hasPluginJson) {
        this.importStatus = 'Failed: no SKILL.md or plugin.json found';
        return;
      }

      const filesDir = getContext(this).filesDir;
      const destBase = hasPluginJson ?
        filesDir + '/.unify/plugins/' + folderName :
        filesDir + '/.unify/skills/' + folderName;
      fileIo.mkdirSync(destBase, true);
      this.copyDirRecursive(srcUri, destBase);

      this.importStatus = `Imported ${hasPluginJson ? 'plugin' : 'skill'}: ${folderName}`;
      // Immediately reload Rust registries
      RustAgentBridge.getInstance().reloadImports(filesDir);
      this.onSettingsSaved();
    } catch (e) {
      console.error('[Settings] importFromUri failed: ' + JSON.stringify(e));
      this.importStatus = 'Import failed: ' + (typeof e === 'object' ? JSON.stringify(e) : 'unknown');
    }
  }

  async createSkillFromText(): Promise<void> {
    try {
      const filesDir = getContext(this).filesDir;
      const destDir = filesDir + '/.unify/skills/' + this.skillName;
      fileIo.mkdirSync(destDir, true);
      const frontmatter = '---\nname: ' + this.skillName +
        '\ndescription: ' + (this.skillDesc || this.skillName) +
        '\ncategory: utility\n---\n\n' + this.skillBody;
      const file = fileIo.openSync(destDir + '/SKILL.md',
        fileIo.OpenMode.CREATE | fileIo.OpenMode.WRITE_ONLY | fileIo.OpenMode.TRUNC);
      fileIo.writeSync(file.fd, frontmatter);
      fileIo.closeSync(file);
      this.importStatus = 'Created skill: ' + this.skillName;
      this.skillName = '';
      this.skillDesc = '';
      this.skillBody = '';
      // Immediately reload Rust skill registry so new skill is available
      RustAgentBridge.getInstance().reloadImports(filesDir);
      // Also trigger ChatView to refresh its cached skill list
      this.onSettingsSaved();
    } catch (e) {
      console.error('[Settings] createSkill failed: ' + JSON.stringify(e));
      this.importStatus = 'Failed to create skill';
    }
  }

  copyDirRecursive(srcDir: string, destDir: string): void {
    try {
      fileIo.mkdirSync(destDir, true);
      const files = fileIo.listFileSync(srcDir);
      for (const f of files) {
        const srcPath = srcDir + '/' + f;
        const destPath = destDir + '/' + f;
        try {
          const stat = fileIo.statSync(srcPath);
          if (stat.isDirectory()) {
            this.copyDirRecursive(srcPath, destPath);
          } else {
            const srcFile = fileIo.openSync(srcPath, fileIo.OpenMode.READ_ONLY);
            const buf = new ArrayBuffer(stat.size);
            fileIo.readSync(srcFile.fd, buf);
            fileIo.closeSync(srcFile);
            const dstFile = fileIo.openSync(destPath,
              fileIo.OpenMode.CREATE | fileIo.OpenMode.WRITE_ONLY | fileIo.OpenMode.TRUNC);
            fileIo.writeSync(dstFile.fd, buf);
            fileIo.closeSync(dstFile);
          }
        } catch (_e) { /* skip individual file errors */ }
      }
    } catch (_e) { /* skip directory errors */ }
  }

  async pickWorkspaceDir(): Promise<void> {
    try {
      const options = new picker.DocumentSelectOptions();
      options.maxSelectNumber = 1;
      // No authMode — let user navigate to any folder, pick any file inside it
      const docPicker = new picker.DocumentViewPicker(getContext(this));
      const uris = await docPicker.select(options);
      if (uris && uris.length > 0) {
        const uri = uris[0];
        const lastSlash = uri.lastIndexOf('/');
        const parentUri = lastSlash > 0 ? uri.substring(0, lastSlash) : uri;
        const prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
        await prefs.put('workspace_uri', parentUri);
        await prefs.put('workspace_mode', 'rw');
        await prefs.flush();
        this.importStatus = 'Workspace: ' + parentUri;
      }
    } catch (e) {
      console.error('[Settings] pickWorkspace failed: ' + JSON.stringify(e));
      this.importStatus = 'Workspace select failed';
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
    Scroll() {
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

      // ── Import Skills/Plugins ──
      Text('Skills & Workspace').fontSize(12).fontColor('#999').margin({ top: 20, bottom: 4 })

      Button('Import from Device')
        .fontSize(13).width('100%').height(36)
        .backgroundColor('#e0e0e0').fontColor('#333')
        .onClick(() => this.pickAndImportFolder())

      Button('Select Workspace Dir')
        .fontSize(13).width('100%').height(36)
        .backgroundColor('#e0e0e0').fontColor('#333')
        .margin({ top: 4 })
        .onClick(() => this.pickWorkspaceDir())

      if (this.importStatus !== '') {
        Text(this.importStatus)
          .fontSize(11)
          .fontColor(this.importStatus.indexOf('Failed') >= 0 ? '#e74c3c' : '#4CAF50')
          .margin({ top: 4 })
      }

      // ── Quick Skill Create ──
      Text('Or Create Skill').fontSize(12).fontColor('#999').margin({ top: 12, bottom: 4 })
      TextInput({ text: $$this.skillName, placeholder: 'Skill name (e.g. my-skill)' })
        .fontSize(12).width('100%').height(32)
      TextInput({ text: $$this.skillDesc, placeholder: 'Description' })
        .fontSize(12).width('100%').height(32).margin({ top: 4 })
      TextInput({ text: $$this.skillBody, placeholder: 'Skill body (instructions for AI)' })
        .fontSize(12).width('100%').height(60).margin({ top: 4 })
      Button('Create')
        .fontSize(13).width('100%').height(36)
        .backgroundColor('#e0e0e0').fontColor('#333')
        .onClick(() => this.pickAndImportFolder())

      Button(this.saved ? '✓ Saved' : 'Save Configuration')
        .fontSize(14)
        .onClick(() => this.saveConfig())
    }
    .width('100%')
    .padding(14)
  }
  .width('100%')
  .scrollable(ScrollDirection.Vertical)
  .scrollBar(BarState.Auto)
  }
}
