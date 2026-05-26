import { ArkMessage, ArkContentPart } from '../models';
import { MessageBubble } from './MessageBubble';
import { InputBar } from '../input';
import { RustAgentBridge } from '../../RustAgentBridge';
import { SettingsPage } from '../../pages/SettingsPage';
import { preferences } from '@kit.ArkData';
import { http } from '@kit.NetworkKit';
import { SseStreamController } from '../../stream/SseStreamController';

interface ChatPayload {
  role: string;
  content: string;
}

@ComponentV2
export struct ChatView {
  @Param onBridgeCall: (action: string, args: string) => string =
    (_action: string, _args: string): string => '{}';
  @Local messages: ArkMessage[] = [];
  @Local isStreaming: boolean = false;
  @Local showSettings: boolean = false;
  @Local bridgeReady: boolean = false;
  private bridge: RustAgentBridge = RustAgentBridge.getInstance();
  private streamCtrl: SseStreamController | null = null;

  aboutToAppear(): void {
    this.initBridge();
  }

  async initBridge(): Promise<void> {
    if (this.bridgeReady) {
      return;
    }
    try {
      const prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
      const apiKey = await prefs.get('api_key', '') as string;
      if (!apiKey || apiKey === '') {
        return;
      }
      const baseUrl = await prefs.get('base_url', '') as string;
      const model = await prefs.get('model', 'gpt-4o') as string;
      const provider = await prefs.get('provider', 'openai') as string;

      // Init bridge with streaming callback (host has called initWithIo by now)
      const ok = this.bridge.init((data: string, eventType: number) => {
        if (eventType === 0) {
          this.applyDelta(data);
        } else if (eventType === 1) {
          this.isStreaming = false;
        } else if (eventType === 2) {
          this.isStreaming = false;
        }
      });

      if (ok) {
        this.bridgeReady = true;
        this.bridge.call('configure', JSON.stringify({
          provider: provider,
          api_key: apiKey,
          base_url: baseUrl,
          model: model,
          max_tokens: 32000
        }));
        console.info('[ChatView] Bridge initialized OK');
      }
    } catch (e) {
      console.error('[ChatView] Bridge init failed: ' + e);
    }
  }

  async applyConfig(): Promise<void> {
    try {
      const prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
      const apiKey = await prefs.get('api_key', '') as string;
      if (!apiKey || apiKey === '') {
        return;
      }
      const baseUrl = await prefs.get('base_url', '') as string;
      const model = await prefs.get('model', 'gpt-4o') as string;
      const provider = await prefs.get('provider', 'openai') as string;

      this.bridge.call('configure', JSON.stringify({
        provider: provider,
        api_key: apiKey,
        base_url: baseUrl,
        model: model,
        max_tokens: 32000
      }));
      console.info('[ChatView] Config applied');
    } catch (e) {
      console.error('[ChatView] Config apply failed: ' + e);
    }
  }

  async handleSettingsSaved(): Promise<void> {
    this.showSettings = false;
    await this.initBridge();
    await this.applyConfig();
  }

  applyDelta(deltaJson: string): void {
    try {
      const delta = JSON.parse(deltaJson) as Record<string, string>;
      const last = this.messages.length > 0
        ? this.messages[this.messages.length - 1]
        : null;
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        this.messages.push(msg);
      }
      const cur = this.messages[this.messages.length - 1];
      const tp = delta['type'] as string;
      let part: ArkContentPart | null = null;
      for (let i = cur.parts.length - 1; i >= 0; i--) {
        if (cur.parts[i].type === tp) {
          part = cur.parts[i];
          break;
        }
      }
      if (!part) {
        part = new ArkContentPart(tp as 'text' | 'reasoning' | 'tool_call' | 'tool_result');
        cur.parts.push(part);
        cur.parts = [...cur.parts];
      }
      if (delta['text']) part.text += delta['text'] as string;
      if (delta['collapsed'] !== undefined) part.collapsed = delta['collapsed'] === 'true' || delta['collapsed'] === '1';
      if (delta['id']) part.toolId = delta['id'] as string;
      if (delta['name']) part.name = delta['name'] as string;
      if (delta['is_error'] !== undefined) part.isError = delta['is_error'] === 'true' || delta['is_error'] === '1';
    } catch (_e) {
      const last = this.messages.length > 0 ? this.messages[this.messages.length - 1] : null;
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        this.messages.push(msg);
      }
      const cur = this.messages[this.messages.length - 1];
      let textPart: ArkContentPart | null = null;
      for (let i = cur.parts.length - 1; i >= 0; i--) {
        if (cur.parts[i].type === 'text') { textPart = cur.parts[i]; break; }
      }
      if (!textPart) {
        textPart = new ArkContentPart('text');
        cur.parts.push(textPart);
        cur.parts = [...cur.parts];
      }
      textPart.text += deltaJson;
    }
  }

  addUserMessage(text: string): void {
    const msg = new ArkMessage('user');
    msg.parts = [new ArkContentPart('text')];
    msg.parts[0].text = text;
    this.messages.push(msg);
  }

  handleSend(text: string): void {
    if (!this.bridgeReady) {
      this.addUserMessage(text);
      const reply = new ArkMessage('assistant');
      reply.parts = [new ArkContentPart('text')];
      reply.parts[0].text = 'Please configure API settings first (gear icon above).';
      this.messages.push(reply);
      return;
    }
    this.addUserMessage(text);
    this.bridge.resetAbort();
    this.isStreaming = true;
    console.info('[ChatView] send via Rust/ureq');

    // Build conversation history
    const msgs: ChatPayload[] = [];
    for (const m of this.messages) {
      if (m.role === 'user' || m.role === 'assistant') {
        let content = '';
        for (const p of m.parts) {
          if (p.type === 'text') content += p.text;
        }
        if (content) {
          msgs.push({ role: m.role, content: content });
        }
      }
    }

    // Rust ureq path: bridge.call → json_router → thread → ureq → SSE → stream cb → applyDelta
    try {
      const res = this.bridge.call('chat', JSON.stringify({ messages: msgs }));
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'error') {
        this.isStreaming = false;
        const reply = new ArkMessage('assistant');
        reply.parts = [new ArkContentPart('text')];
        reply.parts[0].text = `Error: ${parsed['error'] || 'Unknown error'}`;
        this.messages.push(reply);
      }
    } catch (e) {
      this.isStreaming = false;
      const reply = new ArkMessage('assistant');
      reply.parts = [new ArkContentPart('text')];
      reply.parts[0].text = `Chat failed: ${e}`;
      this.messages.push(reply);
    }
  }

  handleStop(): void {
    if (this.bridgeReady) {
      this.bridge.abort();
    }
    this.isStreaming = false;
  }

  build() {
    Column() {
      // Top bar with title and settings gear
      Row() {
        Text('AI Assistant')
          .fontSize(14)
          .fontWeight(FontWeight.Medium)
          .fontColor('#333')
        Blank()
        Button('⚙')
          .fontSize(16)
          .width(32).height(32)
          .borderRadius(16)
          .backgroundColor('#f0f0f0')
          .onClick(() => { this.showSettings = !this.showSettings; })
      }
      .width('100%')
      .padding({ left: 12, right: 12, top: 6, bottom: 4 })

      if (this.showSettings) {
        SettingsPage({ onSettingsSaved: () => this.handleSettingsSaved() })
          .height('100%')
      } else {
        List() {
          ForEach(this.messages, (msg: ArkMessage, idx: number) => {
            ListItem() {
              MessageBubble({ message: msg })
            }
          }, (msg: ArkMessage, idx: number) => msg.id || `${idx}`)
        }
        .layoutWeight(1)
        .width('100%')
      }

      InputBar({
        onSend: (text: string) => this.handleSend(text),
        onStop: () => this.handleStop(),
        isStreaming: this.isStreaming
      })
    }
    .width('100%')
    .height('100%')
  }
}
