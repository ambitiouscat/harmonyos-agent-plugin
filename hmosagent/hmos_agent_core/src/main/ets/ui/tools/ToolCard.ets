@ComponentV2
export struct ToolCard {
  @Param @Require toolName: string = '';
  @Param @Require toolId: string = '';
  @Param isError: boolean = false;
  @Param output: string = '';
  @Param collapsed: boolean = true;

  build() {
    Column() {
      Row() {
        // Status indicator
        Circle({ width: 8, height: 8 })
          .fill(this.isError ? '#e74c3c' : (this.output ? '#27ae60' : '#f39c12'))

        Text(this.toolName)
          .fontSize(13)
          .fontWeight(FontWeight.Medium)
          .fontColor('#333')
          .margin({ left: 8 })

        Blank()

        Text(this.collapsed ? '▶' : '▼')
          .fontSize(12)
          .fontColor('#999')
      }
      .width('100%')

      if (!this.collapsed && this.output) {
        Text(this.output)
          .fontSize(12)
          .fontColor('#555')
          .maxLines(10)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .margin({ top: 8 })
          .padding(8)
          .backgroundColor('#fafafa')
          .borderRadius(4)
      }
    }
    .padding(10)
    .backgroundColor('#f8f8f8')
    .borderRadius(8)
    .border({ width: 1, color: '#e0e0e0' })
    .margin({ top: 4, bottom: 4 })
  }
}
