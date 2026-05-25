import { preferences } from '@kit.ArkData';

@ComponentV2
export struct SettingsPage {
  @Local apiKey: string = '';
  @Local baseUrl: string = '';
  @Local temperature: number = 0.7;
  @Local maxTokens: number = 4096;
  @Local saved: boolean = false;

  private prefs: preferences.Preferences | null = null;

  aboutToAppear(): void {
    this.loadConfig();
  }

  async loadConfig(): Promise<void> {
    try {
      this.prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
      this.apiKey = await this.prefs.get('api_key', '') as string;
      this.baseUrl = await this.prefs.get('base_url', '') as string;
      this.temperature = await this.prefs.get('temperature', 0.7) as number;
      this.maxTokens = await this.prefs.get('max_tokens', 4096) as number;
    } catch (_e) {
      // defaults are fine
    }
  }

  async saveConfig(): Promise<void> {
    if (!this.prefs) {
      return;
    }
    try {
      await this.prefs.put('api_key', this.apiKey);
      await this.prefs.put('base_url', this.baseUrl);
      await this.prefs.put('temperature', this.temperature);
      await this.prefs.put('max_tokens', this.maxTokens);
      await this.prefs.flush();
      this.saved = true;
      setTimeout(() => { this.saved = false; }, 2000);
    } catch (_e) {
      // save failed silently
    }
  }

  build() {
    Column({ space: 16 }) {
      Text('Settings')
        .fontSize(22)
        .fontWeight(FontWeight.Bold)
        .margin({ top: 16 })

      // API Key
      Column() {
        Text('API Key').fontSize(13).fontColor('#999')
        TextInput({ text: $$this.apiKey, placeholder: 'sk-...' })
          .type(InputType.Password)
          .fontSize(14)
      }
      .width('100%')

      // Base URL
      Column() {
        Text('Base URL').fontSize(13).fontColor('#999')
        TextInput({ text: $$this.baseUrl, placeholder: 'https://api.openai.com/v1' })
          .fontSize(14)
      }
      .width('100%')

      // Temperature
      Column() {
        Row() {
          Text(`Temperature: ${this.temperature.toFixed(1)}`).fontSize(14)
        }
        Slider({
          value: $$this.temperature,
          min: 0.0,
          max: 2.0,
          step: 0.1
        })
      }
      .width('100%')

      // Max Tokens
      Column() {
        Row() {
          Text(`Max Tokens: ${this.maxTokens}`).fontSize(14)
        }
        Slider({
          value: $$this.maxTokens,
          min: 256,
          max: 32768,
          step: 256
        })
      }
      .width('100%')

      Button(this.saved ? '✓ Saved' : 'Save Configuration')
        .fontSize(16)
        .onClick(() => this.saveConfig())
    }
    .width('100%')
    .padding(20)
  }
}
