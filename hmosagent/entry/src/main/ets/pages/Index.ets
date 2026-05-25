import { RustAgentBridge, ChatView, ArkMessage, ArkContentPart } from 'hmos_agent_core';

@Entry
@Component
struct Index {
  @State @Watch('onMessagesChange') messages: ArkMessage[] = [];
  @State isStreaming: boolean = false;
  @State lastUserInput: string = '';
  private bridge: RustAgentBridge = new RustAgentBridge();
  private nextContent: string = '';

  aboutToAppear(): void {
    this.bridge.init((data: string, eventType: number) => {
      if (eventType === 0 && data) {
        this.applyDelta(data);
      } else if (eventType === 1) {
        this.isStreaming = false;
      }
    });
  }

  onMessagesChange(): void {
    // @Watch callback — force refresh for V1 compatibility
  }

  applyDelta(deltaJson: string): void {
    try {
      const delta = JSON.parse(deltaJson) as Record<string, string>;
      const msgs = this.messages;
      const last = msgs.length > 0 ? msgs[msgs.length - 1] : null;
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        msgs.push(msg);
      }
      const cur = msgs[msgs.length - 1];
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
      if (delta['text']) {
        part.text += delta['text'] as string;
      }
      if (delta['id']) {
        part.toolId = delta['id'] as string;
      }
      if (delta['name']) {
        part.name = delta['name'] as string;
      }
      if (delta['is_error'] !== undefined) {
        part.isError = delta['is_error'] === 'true';
      }
      // Force UI refresh
      this.messages = [...msgs];
    } catch (_e) {
      this.nextContent += deltaJson;
    }
  }

  sendStreamTest(): void {
    this.messages = [];
    const userMsg = new ArkMessage('user');
    userMsg.parts = [new ArkContentPart('text')];
    userMsg.parts[0].text = 'Stream test';
    this.messages = [userMsg];
    this.isStreaming = true;
    this.bridge.call('test_stream', '{"chunks":12,"interval_ms":80}');
  }

  build() {
    Column() {
      Row() {
        Text('HmosAgent').fontSize(18).fontWeight(FontWeight.Bold)
        Blank()
        Button('Test Stream')
          .fontSize(13)
          .height(36)
          .onClick(() => this.sendStreamTest())
      }
      .width('100%')
      .padding({ left: 16, right: 16, top: 8 })

      Column() {
        ForEach(this.messages, (msg: ArkMessage, idx: number) => {
          Column() {
            Row() {
              Text(msg.role === 'user' ? 'You' : 'AI')
                .fontSize(11).fontColor('#999')
            }
            .width('100%')

            ForEach(msg.parts, (part: ArkContentPart) => {
              if (part.type === 'text') {
                Text(part.text)
                  .fontSize(14)
                  .lineHeight(20)
                  .padding({ top: 2, bottom: 2 })
              } else if (part.type === 'reasoning') {
                Text(`[Think] ${part.text}`)
                  .fontSize(12).fontColor('#888').fontStyle(FontStyle.Italic)
              } else if (part.type === 'tool_call') {
                Text(`[Tool: ${part.name}]`)
                  .fontSize(12).fontColor('#2196F3')
              } else if (part.type === 'tool_result') {
                Text(`[Result] ${part.text}`)
                  .fontSize(12).fontColor(part.isError ? '#e74c3c' : '#27ae60')
              }
            })
          }
          .width('100%')
          .padding(12)
          .alignItems(msg.role === 'user' ? HorizontalAlign.End : HorizontalAlign.Start)
        }, (msg: ArkMessage, idx: number) => msg.id || `${idx}`)
      }
      .layoutWeight(1)
      .width('100%')

      Row() {
        TextInput({ placeholder: 'Type a message...' })
          .layoutWeight(1)
          .height(40)
          .fontSize(14)
          .backgroundColor('#f5f5f5')
          .borderRadius(20)
          .padding({ left: 16, right: 16 })

        Button('↑')
          .fontSize(18)
          .fontColor(Color.White)
          .backgroundColor('#2196F3')
          .borderRadius(20)
          .width(40)
          .height(40)
          .margin({ left: 8 })
          .onClick(() => this.sendStreamTest())
      }
      .width('100%')
      .padding({ left: 12, right: 12, top: 8, bottom: 8 })
      .backgroundColor('#ffffff')
    }
    .width('100%')
    .height('100%')
  }
}
