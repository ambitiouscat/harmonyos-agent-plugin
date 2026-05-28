/**
 * ToolProgressBar — compact tool execution indicator.
 * Shows the currently running tool name with a pulsing dot,
 * displayed next to the phrase spinner in a shared row.
 */
@ComponentV2
export struct ToolProgressBar {
  @Param toolName: string = '';
  @Param toolArgs: string = '';
  @Param visible: boolean = false;
  @Param color: string = '#FFD700';

  build() {
    if (this.visible && this.toolName) {
      Row() {
        // Pulsing dot
        Circle()
          .width(7)
          .height(7)
          .fill(this.color)

        Text(this.toolName)
          .fontSize(14)
          .fontColor(this.color)
          .fontWeight(FontWeight.Bold)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }
      .padding({ left: 8, right: 8, top: 2, bottom: 2 })
      .borderRadius(10)
      .backgroundColor(this.color + '18')
    }
  }
}
