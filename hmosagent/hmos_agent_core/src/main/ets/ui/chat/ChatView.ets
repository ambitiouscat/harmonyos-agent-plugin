import { ArkMessage } from '../models/ArkMessage';
import { ArkContentPart } from '../models/ArkContentPart';
import { MessageBubble } from './MessageBubble';
import { InputBar } from '../input/InputBar';

@ComponentV2
export struct ChatView {
  @Param onBridgeCall: (action: string, args: string) => string =
    (_action: string, _args: string): string => '{}';
  @Local messages: ArkMessage[] = [];
  @Local isStreaming: boolean = false;

  // Apply a ContentPart JSON delta to the current assistant message.
  applyDelta(deltaJson: string): void {
    try {
      const delta = JSON.parse(deltaJson) as Record<string, string>;
      const last = this.messages.length > 0
        ? this.messages[this.messages.length - 1]
        : null;

      // If a new message is needed, create one.
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        this.messages.push(msg);
      }

      const cur = this.messages[this.messages.length - 1];
      const tp = delta['type'] as string;

      // Try to append to an existing part of the same type.
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
        // Trigger array change for @Trace
        cur.parts = [...cur.parts];
      }

      // Merge delta fields.
      if (delta['text']) {
        part.text += delta['text'] as string;
      }
      if (delta['collapsed'] !== undefined) {
        part.collapsed = delta['collapsed'] === 'true' || delta['collapsed'] === '1';
      }
      if (delta['id']) {
        part.toolId = delta['id'] as string;
      }
      if (delta['name']) {
        part.name = delta['name'] as string;
      }
      if (delta['is_error'] !== undefined) {
        part.isError = delta['is_error'] === 'true' || delta['is_error'] === '1';
      }

    } catch (_e) {
      // Non-JSON chunk — append as plain text to last text part.
      const last = this.messages.length > 0
        ? this.messages[this.messages.length - 1]
        : null;
      if (!last || last.role !== 'assistant') {
        const msg = new ArkMessage('assistant');
        this.messages.push(msg);
      }
      const cur = this.messages[this.messages.length - 1];
      let textPart: ArkContentPart | null = null;
      for (let i = cur.parts.length - 1; i >= 0; i--) {
        if (cur.parts[i].type === 'text') {
          textPart = cur.parts[i];
          break;
        }
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
    this.addUserMessage(text);
    this.isStreaming = true;
  }

  handleStop(): void {
    this.isStreaming = false;
  }

  build() {
    Column() {
      List() {
        ForEach(this.messages, (msg: ArkMessage, idx: number) => {
          ListItem() {
            MessageBubble({ message: msg })
          }
        }, (msg: ArkMessage, idx: number) => msg.id || `${idx}`)
      }
      .layoutWeight(1)
      .width('100%')

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
