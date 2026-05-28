/**
 * StatusBar — dynamic status display with auto-wrapping.
 * Items are added/removed dynamically. Fun phrase always first.
 * Auto-expands to multiple lines when one row is insufficient.
 */

export interface StatusItem {
  id: string;
  label: string;
  color: string;
}

@ComponentV2
export struct StatusBar {
  @Param items: StatusItem[] = [];
  @Param visible: boolean = false;

  build() {
    if (this.visible && this.items.length > 0) {
      Flex({ wrap: FlexWrap.Wrap }) {
        ForEach(this.items, (item: StatusItem, idx: number) => {
          Row() {
            // Dot indicator
            Circle()
              .width(6)
              .height(6)
              .fill(item.color)

            Text(item.label)
              .fontSize(13)
              .fontColor(item.color)
              .fontWeight(FontWeight.Bold)
              .maxLines(1)
              .textOverflow({ overflow: TextOverflow.Ellipsis })
          }
          .padding({ left: 6, right: 10, top: 2, bottom: 2 })
          .borderRadius(8)
          .backgroundColor(item.color + '12')
          .margin({ right: 6, bottom: 3 })
        }, (item: StatusItem) => item.id)
      }
      .width('100%')
      .padding({ left: 12, right: 12, top: 2, bottom: 2 })
    }
  }
}
