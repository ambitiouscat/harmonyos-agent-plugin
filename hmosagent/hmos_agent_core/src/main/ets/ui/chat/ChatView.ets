import { ArkMessage, ArkContentPart } from '../models';
import { MessageBubble } from './MessageBubble';
import { InputBar } from '../input';
import { ConversationList } from '../conversation';
import { RustAgentBridge } from '../../RustAgentBridge';
import { SettingsPage } from '../../pages/SettingsPage';
import { preferences } from '@kit.ArkData';
import { http } from '@kit.NetworkKit';
import { SseStreamController } from '../../stream/SseStreamController';
import { getPhraseLoader, getThemeLoader, ThemeConfig, StatusBar, StatusItem, InfoBar, TaskPanel, PermissionDialog } from '../common';
import { getProviderLoader } from '../../provider/ProviderLoader';

interface ChatPayload {
  role: string;
  content: string;
}

interface SkillItem {
  name: string;
  description: string;
  category: string;
  params: Array<Record<string, string>>;
}

interface SessionItem {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

interface TaskItem {
  id: string;
  content: string;
  status: string;
  blocked_by: string[];
}

@ComponentV2
export struct ChatView {
  @Param onBridgeCall: (action: string, args: string) => string =
    (_action: string, _args: string): string => '{}';
  @Local messages: ArkMessage[] = [];
  @Local isStreaming: boolean = false;
  @Local showSettings: boolean = false;
  @Local bridgeReady: boolean = false;
  @Local statusPhrase: string = '';
  @Local statusElapsed: number = 0;
  @Local statusVisible: boolean = false;
  @Local showDrawer: boolean = false;
  @Local sessions: SessionItem[] = [];
  @Local activeSessionId: string = '';
  @Local activeSessionTitle: string = '';
  @Local skillList: SkillItem[] = [];
  @Local isOffline: boolean = false;
  @Local themeVersion: number = 0;
  @Local sessionPending: boolean = false;
  @Local loadedCount: number = 30;
  @Local hasMoreToLoad: boolean = false;
  // Dynamic Status bar items (fun phrase, tools, etc.)
  @Local statusItems: StatusItem[] = [];
  // Fixed info bar (model, folder, context, memory)
  @Local fixedModelName: string = '';
  @Local fixedFolder: string = '';
  @Local contextUsed: string = '';
  @Local contextMax: string = '';
  @Local memoryUsage: string = '';
  // Task panel + permission
  @Local showTaskPanel: boolean = false;
  @Local taskList: TaskItem[] = [];
  @Local showPermission: boolean = false;
  @Local permToolName: string = '';
  @Local permArgs: string = '';
  @Local permReason: string = '';
  private bridge: RustAgentBridge = RustAgentBridge.getInstance();
  private scroller: Scroller = new Scroller();
  private pageSize: number = 30;
  private lastScrollTime: number = 0;
  private allMessages: ArkMessage[] = [];
  private streamCtrl: SseStreamController | null = null;
  private phraseQueue: string[] = [];
  private phraseTimerId: number = -1;
  private elapsedTimerId: number = -1;
  private phraseStyle: string = 'claude-original';
  private phraseIntervalMin: number = 5000;
  private phraseIntervalMax: number = 10000;
  private loader = getPhraseLoader();
  @Local theme: ThemeConfig = getThemeLoader().getCurrent();
  private themeLoader = getThemeLoader();

  aboutToAppear(): void {
    this.themeLoader.load(getContext(this));
    this.theme = this.themeLoader.getCurrent();
    this.loadAppConfig();
    this.initBridge();
    getPhraseLoader().load(getContext(this));
    this.bridge.onOfflineChange((offline: boolean) => {
      this.isOffline = offline;
    });
    // Restore phrase preference from persistent storage
    this.restorePhraseStyle();
  }

  async restorePhraseStyle(): Promise<void> {
    // Already set via AppStorage? (same-session restart)
    try {
      const fromMem = AppStorage.get<string>('thinking_phrase_style');
      if (fromMem) {
        this.phraseStyle = fromMem;
        return;
      }
    } catch (_e) { /* ignore */ }
    // Read from persistent Preferences on cold start
    try {
      const prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
      const saved = await prefs.get('thinking_phrase_style', 'claude-original') as string;
      AppStorage.setOrCreate('thinking_phrase_style', saved);
      this.phraseStyle = saved;
    } catch (_e) {
      this.phraseStyle = 'claude-original';
    }
  }

  loadAppConfig(): void {
    try {
      const rm = getContext(this).resourceManager;
      const raw = rm.getRawFileContentSync('app_config.json');
      let text = '';
      for (let i = 0; i < raw.length; i++) {
        text += String.fromCharCode(raw[i]);
      }
      const cfg = JSON.parse(text) as Record<string, number>;
      if (cfg['message_page_size'] && cfg['message_page_size'] > 0) {
        this.pageSize = cfg['message_page_size'] as number;
        this.loadedCount = this.pageSize;
      }
      if (cfg['phrase_interval_min_ms'] && cfg['phrase_interval_min_ms'] > 0) {
        this.phraseIntervalMin = cfg['phrase_interval_min_ms'] as number;
      }
      if (cfg['phrase_interval_max_ms'] && cfg['phrase_interval_max_ms'] > 0) {
        this.phraseIntervalMax = cfg['phrase_interval_max_ms'] as number;
      }
    } catch (_e) { /* use defaults */ }
  }

  async initBridge(): Promise<void> {
    if (this.bridgeReady) {
      return;
    }
    try {
      // Init bridge — always works, even without API key
      const ok = this.bridge.init((data: string, eventType: number) => {
        if (eventType === 0) {
          this.applyDelta(data);
        } else if (eventType === 1) {
          this.isStreaming = false;
          this.stopStatusSpinner();
          this.removeStatusItem('tool');
          this.updateContextUsage();
          this.saveCurrentSession();
        } else if (eventType === 2) {
          this.isStreaming = false;
          this.stopStatusSpinner();
          this.removeStatusItem('tool');
          this.updateContextUsage();
          try {
            const errData = JSON.parse(data) as Record<string, string>;
            const errMsg = new ArkMessage('assistant');
            errMsg.parts = [new ArkContentPart('text')];
            errMsg.parts[0].text = `Error: ${errData['text'] || 'Unknown error'}`;
            this.allMessages.push(errMsg);
            this.loadedCount++;
            this.syncVisibleMessages();
          } catch (_e) { /* ignore */ }
        } else if (eventType === 3) {
          // tool_start: add to dynamic status bar
          try {
            const toolData = JSON.parse(data) as Record<string, string>;
            const name = toolData['name'] || '';
            if (name) {
              this.upsertStatusItem('tool', name, this.getToolColor(name));
            }
          } catch (_e) { /* ignore */ }
        }
      });

      if (ok) {
        this.bridgeReady = true;
        // Init session & skills — don't need API key
        const filesDir = getContext(this).filesDir;
        this.bridge.initSession(filesDir);
        this.loadSkills();

        // Set up permission callback
        this.bridge.onPermissionRequest((toolName: string, reason: string) => {
          this.permToolName = toolName;
          this.permReason = reason;
          this.permArgs = '';
          this.showPermission = true;
        });

        // LLM config — optional, only if API key is set
        const prefs = await preferences.getPreferences(getContext(this), 'hmos_agent_config');
        const apiKey = await prefs.get('api_key', '') as string;
        if (apiKey && apiKey !== '') {
          const baseUrl = await prefs.get('base_url', '') as string;
          const model = await prefs.get('model', 'gpt-4o') as string;
          const provider = await prefs.get('provider', 'openai') as string;
          this.fixedModelName = model;
          this.fixedFolder = this.shortFolderName(filesDir);
          const ctxWin = getProviderLoader().getModelContext(provider, model);
          if (ctxWin > 0) {
            this.contextMax = ctxWin >= 1000 ? (ctxWin / 1000).toFixed(0) + 'k' : ctxWin.toString();
          } else {
            this.contextMax = '128k'; // fallback
          }
          this.bridge.call('configure', JSON.stringify({
            provider: provider,
            api_key: apiKey,
            base_url: baseUrl,
            model: model,
            max_tokens: 32000,
            sandbox_root: filesDir
          }));
        }
        console.info('[ChatView] Bridge initialized OK');
        // Auto-init local session (not saved until first message)
        if (!this.activeSessionId) {
          this.activeSessionId = this.genLocalId();
          this.activeSessionTitle = 'Chat ' + new Date().toISOString().substring(0, 16).replace('T', ' ');
          this.sessionPending = true;
        }
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

      this.fixedModelName = model;
      const ctxWin = getProviderLoader().getModelContext(provider, model);
      if (ctxWin > 0) {
        this.contextMax = ctxWin >= 1000 ? (ctxWin / 1000).toFixed(0) + 'k' : ctxWin.toString();
      }
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
      // Reconnect notification from Rust — show status, skip content
      if (delta['type'] === 'reconnect') {
        this.statusPhrase = 'Reconnecting… (attempt ' + (delta['attempt'] || '?') + ')';
        if (!this.statusVisible) {
          this.statusVisible = true;
          this.statusElapsed = 0;
        }
        return;
      }
      const last = this.messages.length > 0
        ? this.messages[this.messages.length - 1]
        : null;
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        this.allMessages.push(msg);
        this.loadedCount++;
        this.syncVisibleMessages();
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
        this.allMessages.push(msg);
        this.loadedCount++;
        this.syncVisibleMessages();
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
    this.scrollToBottom();
  }

  addUserMessage(text: string): void {
    const msg = new ArkMessage('user');
    msg.parts = [new ArkContentPart('text')];
    msg.parts[0].text = text;
    this.allMessages.push(msg);
    this.loadedCount++;
    this.syncVisibleMessages();
    this.scrollToBottom();
  }

  handleSend(text: string): void {
    if (!this.bridgeReady) {
      this.addUserMessage(text);
      const reply = new ArkMessage('assistant');
      reply.parts = [new ArkContentPart('text')];
      reply.parts[0].text = 'Please configure API settings first (gear icon above).';
      this.allMessages.push(reply);
        this.loadedCount++;
        this.syncVisibleMessages();
      return;
    }
    this.addUserMessage(text);
    this.bridge.resetAbort();
    this.isStreaming = true;
    this.updateContextUsage();
    this.startStatusSpinner();
    console.info('[ChatView] send via Rust/ureq');

    // Build conversation history from ALL messages (not just visible window)
    const msgs: ChatPayload[] = [];
    for (const m of this.allMessages) {
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

    // V2 agent_loop: bridge.call → json_router → thread → LLM with tools → tool_use loop → stream cb
    try {
      const res = this.bridge.call('agent_loop', JSON.stringify({ messages: msgs }));
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'error') {
        this.isStreaming = false;
        this.stopStatusSpinner();
        const reply = new ArkMessage('assistant');
        reply.parts = [new ArkContentPart('text')];
        reply.parts[0].text = `Error: ${parsed['error'] || 'Unknown error'}`;
        this.allMessages.push(reply);
        this.loadedCount++;
        this.syncVisibleMessages();
      }
    } catch (e) {
      this.isStreaming = false;
      this.stopStatusSpinner();
      const reply = new ArkMessage('assistant');
      reply.parts = [new ArkContentPart('text')];
      reply.parts[0].text = `Chat failed: ${e}`;
      this.allMessages.push(reply);
        this.loadedCount++;
        this.syncVisibleMessages();
    }
    // Restore input focus after send
    setTimeout(() => {
      focusControl.requestFocus('chat_input_field');
    }, 150);
  }

  handleStop(): void {
    if (this.bridgeReady) {
      this.bridge.abort();
    }
    this.isStreaming = false;
    this.stopStatusSpinner();
  }

  // --- Status bar management ---

  upsertStatusItem(id: string, label: string, color: string): void {
    // Replace existing item with same id, or add new.
    // Phrase (id='phrase') always stays first.
    const filtered = this.statusItems.filter((s: StatusItem) => s.id !== id);
    if (id === 'phrase') {
      filtered.unshift({ id: id, label: label, color: color });
    } else {
      filtered.push({ id: id, label: label, color: color });
    }
    this.statusItems = filtered;
  }

  removeStatusItem(id: string): void {
    this.statusItems = this.statusItems.filter((s: StatusItem) => s.id !== id);
  }

  shortFolderName(path: string): string {
    const parts = path.split(/[\\/]/);
    // Show last 2 segments for context
    if (parts.length >= 2) return parts[parts.length - 2] + '/' + parts[parts.length - 1];
    return parts[parts.length - 1] || path;
  }

  updateContextUsage(): void {
    // Estimate tokens: ~4 chars per token, each message ~hundreds of chars
    let totalChars = 0;
    for (const m of this.allMessages) {
      for (const p of m.parts) {
        totalChars += p.text.length;
      }
    }
    const estTokens = Math.round(totalChars / 4);
    if (estTokens >= 1000) {
      this.contextUsed = (estTokens / 1000).toFixed(1) + 'k';
    } else {
      this.contextUsed = estTokens.toString();
    }
  }

  startStatusSpinner(): void {
    this.loadPhraseStyle();
    this.phraseQueue = this.loader.getSpinnerPhrases(this.phraseStyle);
    this.statusElapsed = 0;
    this.statusVisible = true;
    if (this.phraseQueue.length === 0) {
      this.phraseQueue = ['Thinking'];
    }
    // Pick random starting phrase
    this.statusPhrase = this.phraseQueue[Math.floor(Math.random() * this.phraseQueue.length)];

    // Add phrase to status bar (id='phrase', always first)
    this.upsertStatusItem('phrase', this.statusPhrase + '…  (' + this.formatElapsed(0) + ')', this.theme.text_muted);

    this.elapsedTimerId = setInterval(() => {
      this.statusElapsed++;
      this.upsertStatusItem('phrase', this.statusPhrase + '…  (' + this.formatElapsed(this.statusElapsed) + ')', this.theme.text_muted);
    }, 1000);

    this.scheduleNextPhrase();
  }

  scheduleNextPhrase(): void {
    const min = this.phraseIntervalMin;
    const max = this.phraseIntervalMax;
    const delay = min + Math.floor(Math.random() * (max - min));
    this.phraseTimerId = setTimeout(() => {
      let next = this.statusPhrase;
      if (this.phraseQueue.length > 1) {
        while (next === this.statusPhrase) {
          next = this.phraseQueue[Math.floor(Math.random() * this.phraseQueue.length)];
        }
      }
      this.statusPhrase = next;
      // Update the status bar item with new phrase
      this.upsertStatusItem('phrase', this.statusPhrase + '…  (' + this.formatElapsed(this.statusElapsed) + ')', this.theme.text_muted);
      if (this.isStreaming) {
        this.scheduleNextPhrase();
      }
    }, delay);
  }

  stopStatusSpinner(): void {
    if (this.phraseTimerId !== -1) {
      clearTimeout(this.phraseTimerId);
      this.phraseTimerId = -1;
    }
    if (this.elapsedTimerId !== -1) {
      clearInterval(this.elapsedTimerId);
      this.elapsedTimerId = -1;
    }
    this.statusVisible = false;
    this.removeStatusItem('phrase');
  }

  formatElapsed(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  loadPhraseStyle(): void {
    try {
      const val = AppStorage.get<string>('thinking_phrase_style');
      if (val) {
        this.phraseStyle = val;
      }
    } catch (_e) {
      this.phraseStyle = 'claude-original';
    }
  }

  // --- Skills ---

  loadSkills(): void {
    if (!this.bridgeReady) return;
    const res = this.bridge.call('get_registered_skills', '{}');
    try {
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'ok' && parsed['message']) {
        this.skillList = JSON.parse(parsed['message']) as SkillItem[];
      }
    } catch (_e) { /* ignore */ }
  }

  toggleTheme(): void {
    this.themeLoader.cycleNext();
    this.theme = this.themeLoader.getCurrent();
    this.themeVersion++;
  }

  // --- Session management ---

  loadSessions(): void {
    if (!this.bridgeReady) return;
    const res = this.bridge.call('list_sessions', '{}');
    try {
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'ok' && parsed['message']) {
        this.sessions = JSON.parse(parsed['message']) as SessionItem[];
      }
    } catch (_e) { /* ignore */ }
  }

  selectSession(id: string): void {
    this.activeSessionId = id;
    this.sessionPending = false;
    this.showDrawer = false;
    // Find title from sessions list
    const found = this.sessions.find((s: SessionItem) => s.id === id);
    this.activeSessionTitle = found ? found.title : '';
    if (!this.bridgeReady) return;
    const res = this.bridge.call('load_session', JSON.stringify({ session_id: id }));
    try {
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'ok' && parsed['message']) {
        const session = JSON.parse(parsed['message']) as Record<string, Object>;
        if (session['meta']) {
          const meta = session['meta'] as Record<string, string>;
          this.activeSessionTitle = meta['title'] || this.activeSessionTitle;
        }
        const msgList = session['messages'] as Array<Object>;
        if (msgList && Array.isArray(msgList)) {
          // Store all messages, render only last N
          const all: ArkMessage[] = [];
          for (let j = 0; j < msgList.length; j++) {
            const m = msgList[j] as Record<string, Object>;
            const role: string = (m['role'] as string) || 'user';
            const validRole: 'system' | 'user' | 'assistant' | 'tool' =
              (role === 'system' || role === 'user' || role === 'assistant' || role === 'tool')
                ? role as 'system' | 'user' | 'assistant' | 'tool' : 'user';
            const msg = new ArkMessage(validRole);
            const partsList = m['parts'] as Array<Object>;
            if (partsList && Array.isArray(partsList)) {
              for (let i = 0; i < partsList.length; i++) {
                const p = partsList[i] as Record<string, string>;
                const part = new ArkContentPart((p['type'] || 'text') as 'text' | 'reasoning' | 'tool_call' | 'tool_result');
                part.text = p['text'] || '';
                msg.parts.push(part);
              }
            }
            if (msg.parts.length > 0) {
              all.push(msg);
            }
          }
          this.allMessages = all;
          this.loadedCount = this.pageSize;
          this.hasMoreToLoad = all.length > this.pageSize;
          this.messages = all.slice(Math.max(0, all.length - this.pageSize));
          // Auto-scroll to bottom after load
          setTimeout(() => this.scrollToBottomNow(), 100);
        }
      }
    } catch (_e) { /* ignore */ }
  }

  deleteSession(id: string): void {
    if (!this.bridgeReady) return;
    this.bridge.call('delete_session', JSON.stringify({ session_id: id }));
    this.sessions = this.sessions.filter((s: SessionItem) => s.id !== id);
    if (this.activeSessionId === id) {
      this.activeSessionId = '';
    }
  }

  createSession(): void {
    // Lazy-create: hold locally until first message
    this.activeSessionId = this.genLocalId();
    this.activeSessionTitle = 'Chat ' + new Date().toISOString().substring(0, 16).replace('T', ' ');
    this.sessionPending = true;
    this.allMessages = [];
    this.messages = [];
    this.loadedCount = this.pageSize;
    this.hasMoreToLoad = false;
    this.showDrawer = false;
  }

  private genLocalId(): string {
    return 's_' + Date.now().toString(36) + '_' + Math.random().toString(36).substring(2, 8);
  }

  saveCurrentSession(): void {
    if (!this.bridgeReady || !this.activeSessionId) return;
    if (this.messages.length === 0) return;
    // Lazy-create: first message flushes the pending session to Rust
    if (this.sessionPending) {
      const res = this.bridge.call('create_session',
        JSON.stringify({ title: this.activeSessionTitle }));
      try {
        const parsed = JSON.parse(res) as Record<string, string>;
        if (parsed['status'] === 'ok' && parsed['message']) {
          const s = JSON.parse(parsed['message']) as Record<string, Object>;
          const meta = s['meta'] as Record<string, string>;
          if (meta && meta['id']) {
            this.activeSessionId = meta['id'] as string;
          }
        }
      } catch (_e) { /* keep local ID */ }
      this.sessionPending = false;
    }
    // Build session JSON string directly (avoids arkts-no-untyped-obj-literals)
    let msgsJson = '';
    for (let i = 0; i < this.messages.length; i++) {
      const m = this.messages[i];
      let partsJson = '';
      for (let j = 0; j < m.parts.length; j++) {
        const p = m.parts[j];
        if (j > 0) partsJson += ',';
        partsJson += `{"type":"${p.type}","text":"${this.escapeJson(p.text)}"}`;
      }
      if (i > 0) msgsJson += ',';
      msgsJson += `{"role":"${m.role}","parts":[${partsJson}]}`;
    }
    const sessionJson = `{"meta":{"id":"${this.activeSessionId}","title":"${this.escapeJson(this.activeSessionTitle)}","created_at":"","updated_at":"${new Date().toISOString()}","message_count":${this.messages.length}},"messages":[${msgsJson}]}`;
    this.bridge.call('save_session', sessionJson);
  }

  private escapeJson(s: string): string {
    return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n').replace(/\r/g, '');
  }

  // --- Task list loading ---
  loadTasks(): void {
    if (!this.bridgeReady) return;
    try {
      const res = this.bridge.call('list_tasks', '{}');
      const parsed = JSON.parse(res) as Record<string, string>;
      if (parsed['status'] === 'ok' && parsed['message']) {
        this.taskList = JSON.parse(parsed['message']) as TaskItem[];
      }
    } catch (_e) { /* ignore */ }
  }

  scrollToBottom(): void {
    const now = Date.now();
    if (now - this.lastScrollTime < 250) return;
    this.lastScrollTime = now;
    this.scroller.scrollEdge(Edge.Bottom);
  }

  scrollToBottomNow(): void {
    this.lastScrollTime = 0; // bypass throttle
    this.scroller.scrollEdge(Edge.Bottom);
  }

  syncVisibleMessages(): void {
    // New messages extend the visible window from the bottom
    this.messages = this.allMessages.slice(Math.max(0, this.allMessages.length - this.loadedCount));
  }

  loadMore(): void {
    if (!this.hasMoreToLoad) return;
    const remaining = this.allMessages.length - this.loadedCount;
    if (remaining <= 0) {
      this.hasMoreToLoad = false;
      return;
    }
    const take = Math.min(this.pageSize, remaining);
    this.loadedCount += take;
    this.syncVisibleMessages();
    this.hasMoreToLoad = this.loadedCount < this.allMessages.length;
  }

  // Map tool name to color from theme config, fallback to accent
  getToolColor(toolName: string): string {
    const colors = this.theme.tool_colors as Record<string, string>;
    if (colors && colors[toolName]) {
      return colors[toolName];
    }
    return this.theme.accent;
  }

  build() {
    Stack() {
      Column() {
        // Top bar with hamburger, title and settings gear
        Row() {
          Button() {
            Image($r('app.media.icon_menu'))
              .width(20).height(20)
              .fillColor(this.theme.accent)
          }
            .width(32).height(32)
            .borderRadius(16)
            .backgroundColor(this.theme.card_bg)
            .onClick(() => {
              this.loadSessions();
              this.showDrawer = !this.showDrawer;
            })
          Text(this.activeSessionTitle || 'AI Assistant')
            .fontSize(14)
            .fontWeight(FontWeight.Medium)
            .fontColor(this.theme.text_primary)
            .margin({ left: 8 })
            .maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
          Blank()
          Button() {
            Image($r('app.media.icon_refresh'))
              .width(20).height(20)
              .fillColor(this.theme.accent)
          }
            .width(32).height(32)
            .borderRadius(16)
            .backgroundColor(this.theme.card_bg)
            .margin({ right: 6 })
            .onClick(() => this.toggleTheme())
          // Task panel toggle
          Button() {
            Image($r('app.media.icon_menu'))
              .width(18).height(18)
              .fillColor(this.theme.accent)
          }
            .width(32).height(32)
            .borderRadius(16)
            .backgroundColor(this.theme.card_bg)
            .margin({ right: 6 })
            .onClick(() => {
              this.loadTasks();
              this.showTaskPanel = !this.showTaskPanel;
            })
          Button() {
            Image($r('app.media.icon_settings'))
              .width(20).height(20)
              .fillColor(this.theme.accent)
          }
            .width(32).height(32)
            .borderRadius(16)
            .backgroundColor(this.theme.card_bg)
            .onClick(() => { this.showSettings = !this.showSettings; })
        }
        .width('100%')
        .padding({ left: 12, right: 12, top: 6, bottom: 4 })
        .backgroundColor(this.theme.page_bg)

        // Offline degradation banner
        if (this.isOffline) {
          Row() {
            Image($r('app.media.icon_warning'))
              .width(18).height(18)
              .fillColor('#ffa500')
              .margin({ right: 6 })
            Text('Connection lost — running in offline mode (local RAG + sandbox only)')
              .fontSize(12).fontColor('#ffa500')
              .layoutWeight(1)
              .maxLines(2)
              .textOverflow({ overflow: TextOverflow.Ellipsis })
            Button() {
              Image($r('app.media.icon_retry'))
                .width(16).height(16)
                .fillColor('#1a1d23')
            }
              .backgroundColor('#ffa500')
              .borderRadius(12)
              .width(32).height(32)
              .margin({ left: 8 })
              .onClick(() => {
                this.bridge.resetOffline();
                this.isOffline = false;
              })
          }
          .width('100%')
          .padding({ left: 12, right: 12, top: 6, bottom: 6 })
          .backgroundColor('rgba(255,165,0,0.12)')
          .borderWidth({ bottom: 1 })
          .borderColor('rgba(255,165,0,0.25)')
        }

        Scroll(this.scroller) {
            Column() {
              // Load-more indicator at top
              if (this.hasMoreToLoad) {
                Row() {
                  Text('Load earlier messages...')
                    .fontSize(12)
                    .fontColor(this.theme.accent)
                }
                .width('100%')
                .padding({ top: 8, bottom: 8 })
                .justifyContent(FlexAlign.Center)
                .onClick(() => this.loadMore())
              }
              ForEach(this.messages, (msg: ArkMessage, idx: number) => {
                MessageBubble({
                  message: msg,
                  isStreaming: idx === this.messages.length - 1 ? this.isStreaming : false,
                  isThinking: idx === this.messages.length - 1 ? this.isStreaming : false
                })
              }, (msg: ArkMessage, idx: number) => `${this.themeVersion}_${msg.id || idx}`)
              // Bottom spacer for scroll comfort
              Row().height(20).width('100%')
            }
            .width('100%')
          }
          .scrollable(ScrollDirection.Vertical)
          .scrollBar(BarState.Off)
          .edgeEffect(EdgeEffect.Spring)
          .layoutWeight(1)
          .width('100%')
          .backgroundColor(this.theme.page_bg)

        // Dynamic status bar — fun phrase, tools, etc. Auto-wraps to multiple lines.
        StatusBar({
          items: this.statusItems,
          visible: this.statusVisible || this.statusItems.length > 0
        })

        // Fixed info bar — model, folder, context, memory. Right above input.
        InfoBar({
          modelName: this.fixedModelName,
          currentFolder: this.fixedFolder,
          contextUsed: this.contextUsed,
          contextMax: this.contextMax,
          memoryUsage: this.memoryUsage,
          textColor: this.theme.text_muted
        })

        InputBar({
          onSend: (text: string) => this.handleSend(text),
          onStop: () => this.handleStop(),
          isStreaming: this.isStreaming,
          skills: this.skillList,
          themeColor: this.theme.id
        })
      }
      .width('100%')
      .height('100%')

      // Conversation drawer with asymmetric slide transition
      if (this.showDrawer) {
        ConversationList({
          isOpen: this.showDrawer,
          sessions: this.sessions,
          activeSessionId: this.activeSessionId,
          onSelect: (id: string) => this.selectSession(id),
          onDelete: (id: string) => this.deleteSession(id),
          onCreate: () => this.createSession(),
          onClose: () => { this.showDrawer = false; }
        })
          .transition(TransitionEffect.asymmetric(
            // Entry: slide in from left, fast
            TransitionEffect.translate({ x: -2000 })
              .animation({ duration: 220, curve: Curve.FastOutSlowIn }),
            // Exit: slide out to left, complete before unmount
            TransitionEffect.translate({ x: -2000 })
              .animation({ duration: 180, curve: Curve.FastOutLinearIn })
          ))
      }

      // Task panel overlay — slides from right
      if (this.showTaskPanel) {
        TaskPanel({
          isOpen: this.showTaskPanel,
          tasks: this.taskList,
          onClose: () => { this.showTaskPanel = false; }
        })
      }

      // Permission dialog overlay
      if (this.showPermission) {
        PermissionDialog({
          visible: this.showPermission,
          toolName: this.permToolName,
          toolArgs: this.permArgs,
          reason: this.permReason,
          onAllow: () => {
            this.bridge.resolvePermission(true);
            this.showPermission = false;
          },
          onDeny: () => {
            this.bridge.resolvePermission(false);
            this.showPermission = false;
          }
        })
      }

      // Settings overlay — floats above chat, never unmounts the Scroll
      if (this.showSettings) {
        Column() {
          Row() {
            Text('Settings')
              .fontSize(18).fontWeight(FontWeight.Bold)
              .fontColor(this.theme.text_primary)
            Blank()
            Button() {
              Image($r('app.media.icon_close'))
                .width(20).height(20)
                .fillColor(this.theme.text_muted)
            }
            .width(36).height(36)
            .borderRadius(18)
            .backgroundColor(this.theme.card_bg)
            .onClick(() => { this.showSettings = false; })
          }
          .width('100%')
          .padding({ left: 16, right: 16, top: 8, bottom: 8 })
          .backgroundColor(this.theme.card_bg)

          SettingsPage({ onSettingsSaved: () => this.handleSettingsSaved() })
            .layoutWeight(1)
        }
        .width('100%').height('100%')
        .backgroundColor(this.theme.page_bg)
      }
    }
    .width('100%')
    .height('100%')
  }
}
