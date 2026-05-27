import { MarkdownParser, MarkdownBlock } from './MarkdownParser';
import { pasteboard } from '@kit.BasicServicesKit';
import { getThemeLoader, ThemeConfig } from '../common';

@ComponentV2
export struct MarkdownView {
  @Param @Require content: string = '';
  @Param isStreaming: boolean = false;
  @Local showCursor: boolean = false;
  private cursorTimer: number = -1;

  aboutToAppear(): void {
    this.cursorTimer = setInterval(() => {
      this.showCursor = !this.showCursor;
    }, 500);
  }

  aboutToDisappear(): void {
    if (this.cursorTimer !== -1) {
      clearInterval(this.cursorTimer);
      this.cursorTimer = -1;
    }
  }

  private tc(): ThemeConfig {
    return getThemeLoader().getCurrent();
  }

  build() {
    Column() {
      ForEach(MarkdownParser.parse(this.content), (block: MarkdownBlock, idx: number) => {
        Column() {
          if (block.type === 'heading') {
            Text(block.text || '')
              .fontSize(block.level === 1 ? 22 : block.level === 2 ? 18 : 16)
              .fontWeight(FontWeight.Bold)
              .fontColor(this.tc().text_primary)
              .margin({ top: 8, bottom: 4 })
          } else if (block.type === 'paragraph') {
            Row() {
              Text(block.text || '')
                .fontSize(14)
                .lineHeight(20)
                .fontColor(this.tc().text_secondary)
                .layoutWeight(1)
              // Blinking cursor on last paragraph during streaming
              if (this.isStreaming && idx === MarkdownParser.parse(this.content).length - 1 &&
                  this.showCursor && (block.text || '').length > 0) {
                Text('|')
                  .fontSize(14)
                  .fontColor(this.tc().accent)
              }
            }
            .width('100%')
          } else if (block.type === 'bullet_list') {
            Column() {
              ForEach(block.items || [], (item: string) => {
                Row() {
                  Text('  •  ')
                    .fontSize(14)
                    .fontColor(this.tc().text_muted)
                  Text(item)
                    .fontSize(14)
                    .fontColor(this.tc().text_secondary)
                }
                .width('100%')
              })
            }
          } else if (block.type === 'code_block') {
            Column() {
              Row() {
                Text(block.language || 'code')
                  .fontSize(11)
                  .fontColor(this.tc().text_muted)
                Blank()
                Image($r('app.media.icon_copy'))
                  .width(16).height(16)
                  .fillColor(this.tc().text_muted)
                  .onClick(() => {
                    const data = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN,
                      block.text || '');
                    pasteboard.getSystemPasteboard().setData(data);
                  })
              }
              .width('100%')
              .padding({ left: 12, right: 12, top: 6, bottom: 6 })
              .backgroundColor(this.tc().code_header_bg)

              Text(block.text || '')
                .fontSize(12)
                .fontColor(this.tc().code_text)
                .fontFamily('monospace')
                .padding(12)
            }
            .backgroundColor(this.tc().code_bg)
            .borderRadius(8)
            .margin({ top: 4, bottom: 4 })
          }
        }
        .transition(TransitionEffect.OPACITY
          .animation({ duration: 200, curve: Curve.EaseOut }))
      })
    }
    .width('100%')
    .alignItems(HorizontalAlign.Start)
  }
}
