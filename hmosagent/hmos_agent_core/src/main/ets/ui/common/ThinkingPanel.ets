@ComponentV2
export struct ThinkingPanel {
  @Param @Require content: string = '';
  @Param collapsed: boolean = true;

  build() {
    Column() {
      Row() {
        // Left accent bar
        Rect()
          .width(3)
          .height('100%')
          .fill('#888')

        Column() {
          Row() {
            Text('Thinking')
              .fontSize(12)
              .fontColor('#666')
            Blank()
            Text(this.collapsed ? '▶' : '▼')
              .fontSize(12)
              .fontColor('#999')
          }
          .width('100%')
          .padding({ bottom: 4 })

          if (!this.collapsed) {
            Text(this.content)
              .fontSize(13)
              .fontColor('#555')
              .lineHeight(18)
          }
        }
        .layoutWeight(1)
        .padding({ left: 8 })
      }
      .width('100%')
      .alignItems(VerticalAlign.Top)
    }
    .padding(12)
    .backgroundColor('#f0f0f020')
    .borderRadius(8)
  }
}
