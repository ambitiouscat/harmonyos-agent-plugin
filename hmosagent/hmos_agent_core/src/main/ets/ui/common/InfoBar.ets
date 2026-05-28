/**
 * InfoBar — fixed status info row right above the input bar.
 * Shows model, current folder, context usage, memory usage.
 * Auto-wraps to multiple lines when one row is insufficient.
 */

@ComponentV2
export struct InfoBar {
  @Param modelName: string = '';
  @Param currentFolder: string = '';
  @Param contextUsed: string = '';
  @Param contextMax: string = '';
  @Param memoryUsage: string = '';
  @Param textColor: string = '#888';

  build() {
    Flex({ wrap: FlexWrap.Wrap }) {
      if (this.modelName) {
        this.infoChip('M', this.modelName)
      }
      if (this.currentFolder) {
        this.infoChip('F', this.currentFolder)
      }
      if (this.contextUsed && this.contextMax) {
        this.infoChip('C', this.contextUsed + '/' + this.contextMax)
      }
      if (this.memoryUsage) {
        this.infoChip('R', this.memoryUsage)
      }
    }
    .width('100%')
    .padding({ left: 12, right: 12, top: 1, bottom: 3 })
  }

  @Builder
  infoChip(icon: string, text: string) {
    Row() {
      Text(icon)
        .fontSize(11)
        .fontColor(this.textColor)
        .fontWeight(FontWeight.Bold)
        .margin({ right: 3 })

      Text(text)
        .fontSize(12)
        .fontColor(this.textColor)
        .maxLines(1)
        .textOverflow({ overflow: TextOverflow.Ellipsis })
    }
    .padding({ left: 6, right: 8, top: 1, bottom: 1 })
    .border({ width: 1, color: this.textColor + '30', style: BorderStyle.Solid })
    .borderRadius(6)
    .margin({ right: 6, bottom: 2 })
  }
}
