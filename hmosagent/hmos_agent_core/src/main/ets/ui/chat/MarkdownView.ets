import { MarkdownParser, MarkdownBlock } from './MarkdownParser';
import { pasteboard } from '@kit.BasicServicesKit';

@ComponentV2
export struct MarkdownView {
  @Param @Require content: string = '';

  build() {
    Column() {
      ForEach(MarkdownParser.parse(this.content), (block: MarkdownBlock) => {
        if (block.type === 'heading') {
          Text(block.text || '')
            .fontSize(block.level === 1 ? 22 : block.level === 2 ? 18 : 16)
            .fontWeight(FontWeight.Bold)
            .margin({ top: 8, bottom: 4 })
        } else if (block.type === 'paragraph') {
          Text(block.text || '')
            .fontSize(14)
            .lineHeight(20)
        } else if (block.type === 'bullet_list') {
          Column() {
            ForEach(block.items || [], (item: string) => {
              Row() {
                Text('  •  ').fontSize(14)
                Text(item).fontSize(14)
              }
              .width('100%')
            })
          }
        } else if (block.type === 'code_block') {
          Column() {
            Row() {
              Text(block.language || 'code')
                .fontSize(11)
                .fontColor('#999')
              Blank()
              Text('Copy')
                .fontSize(11)
                .fontColor('#666')
                .onClick(() => {
                  const data = pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN,
                    block.text || '');
                  pasteboard.getSystemPasteboard().setData(data);
                })
            }
            .width('100%')
            .padding({ left: 12, right: 12, top: 6, bottom: 6 })
            .backgroundColor('#e0e0e0')

            Text(block.text || '')
              .fontSize(12)
              .fontColor('#333')
              .fontFamily('monospace')
              .padding(12)
          }
          .backgroundColor('#f5f5f5')
          .borderRadius(8)
          .margin({ top: 4, bottom: 4 })
        }
      })
    }
    .width('100%')
    .alignItems(HorizontalAlign.Start)
  }
}
