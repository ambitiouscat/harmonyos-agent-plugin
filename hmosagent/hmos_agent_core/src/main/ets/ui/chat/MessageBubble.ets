import { ArkMessage, ArkContentPart } from '../models';
import { MarkdownView } from './MarkdownView';
import { ThinkingPanel } from '../common';
import { ToolCard } from '../tools';

@ComponentV2
export struct MessageBubble {
  @Param @Require message: ArkMessage = new ArkMessage('user');
  @Param isStreaming: boolean = false;
  @Param isThinking: boolean = false;

  build() {
    Column() {
      Row() {
        Text(this.message.role === 'user' ? 'You' : this.message.role === 'system' ? 'System' : 'AI')
          .fontSize(11)
          .fontColor('#999')
          .margin({ bottom: 4 })
      }
      .width('100%')

      ForEach(this.message.parts, (part: ArkContentPart) => {
        if (part.type === 'text') {
          MarkdownView({ content: part.text })
        } else if (part.type === 'reasoning') {
          ThinkingPanel({ content: part.text, collapsed: part.collapsed, isThinking: this.isThinking, isStreaming: this.isStreaming })
        } else if (part.type === 'tool_call') {
          ToolCard({
            toolName: part.name,
            toolId: part.toolId,
            isError: false,
            output: '',
            collapsed: part.collapsed
          })
        } else if (part.type === 'tool_result') {
          ToolCard({
            toolName: part.name,
            toolId: part.toolId,
            isError: part.isError,
            output: part.text,
            collapsed: part.collapsed
          })
        }
      })
    }
    .width('100%')
    .padding({ left: 12, right: 12, top: 8, bottom: 8 })
    .alignItems(this.message.role === 'user' ? HorizontalAlign.End : HorizontalAlign.Start)
  }
}
