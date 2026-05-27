interface SessionItem {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

@ComponentV2
export struct ConversationList {
  @Param isOpen: boolean = false;
  @Param sessions: SessionItem[] = [];
  @Param activeSessionId: string = '';
  @Param onSelect: (id: string) => void = (_id: string) => {};
  @Param onDelete: (id: string) => void = (_id: string) => {};
  @Param onCreate: () => void = () => {};
  @Param onClose: () => void = () => {};

  build() {
    Stack() {
      // Transparent backdrop — tap to close, no visual overlay
      if (this.isOpen) {
        Column()
          .width('100%').height('100%')
          .onClick(() => this.onClose())
      }

      // Drawer panel — offset below top bar so ☰/⚙ stay reachable
      Column() {
        // Header
        Row() {
          Text('Conversations')
            .fontSize(20).fontWeight(FontWeight.Bold)
            .fontColor('#e0e0e0')
          Blank()
          Button() {
            Image($r('app.media.icon_add'))
              .width(20).height(20)
              .fillColor('#a0c4ff')
          }
            .backgroundColor(Color.Transparent)
            .width(40).height(40)
            .onClick(() => this.onCreate())
          Button() {
            Image($r('app.media.icon_close'))
              .width(20).height(20)
              .fillColor('#888')
          }
            .backgroundColor(Color.Transparent)
            .width(40).height(40)
            .margin({ left: 4 })
            .onClick(() => this.onClose())
        }
        .width('100%').padding(16)
        .borderWidth({ bottom: 1 })
        .borderColor('rgba(255,255,255,0.08)')

        // Session list
        if (this.sessions.length === 0) {
          Column() {
            Text('No conversations yet')
              .fontSize(14).fontColor('#666')
              .margin({ top: 40 })
            Text('Tap + to start a new one')
              .fontSize(12).fontColor('#555')
              .margin({ top: 8 })
          }
          .width('100%').layoutWeight(1)
          .justifyContent(FlexAlign.Center)
        } else {
          List() {
            ForEach(this.sessions, (session: SessionItem) => {
              ListItem() {
                Row() {
                  Column() {
                    Text(session.title || 'Untitled')
                      .fontSize(15).fontColor(this.activeSessionId === session.id ? '#a0c4ff' : '#d0d0d0')
                      .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
                    Text(`${session.message_count} msgs | ${this.formatDate(session.updated_at)}`)
                      .fontSize(11).fontColor('#666')
                      .margin({ top: 4 })
                  }
                  .alignItems(HorizontalAlign.Start)
                  .layoutWeight(1)

                  Button() {
                    Image($r('app.media.icon_delete'))
                      .width(16).height(16)
                      .fillColor('#888')
                  }
                    .backgroundColor(Color.Transparent)
                    .width(32).height(32)
                    .onClick(() => this.onDelete(session.id))
                }
                .width('100%').padding({ left: 16, right: 8, top: 12, bottom: 12 })
                .onClick(() => this.onSelect(session.id))
                .borderRadius(8)
                .backgroundColor(this.activeSessionId === session.id ?
                  'rgba(160,196,255,0.08)' : Color.Transparent)
              }
            }, (session: SessionItem): string => session.id)
          }
          .width('100%').layoutWeight(1)
          .scrollBar(BarState.Off)
          .edgeEffect(EdgeEffect.Spring)
        }
      }
      .width('80%').height('100%')
      .backgroundColor('#1e2130')
      .borderRadius({ topRight: 16, bottomRight: 16 })
    }
    .width('100%').height('100%')
  }

  private formatDate(iso: string): string {
    if (!iso || iso.length < 10) return '';
    return iso.substring(0, 10);
  }
}
