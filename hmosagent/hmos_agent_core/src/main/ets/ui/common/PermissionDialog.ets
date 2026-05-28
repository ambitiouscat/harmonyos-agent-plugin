import { RustAgentBridge } from '../../RustAgentBridge';

/**
 * Permission confirmation dialog for dangerous tool operations.
 *
 * When the Rust core encounters a tool call that requires user approval
 * (Gate 3 of the permission pipeline), it blocks and waits for this dialog.
 * The user has 30 seconds to respond before auto-deny kicks in.
 */
@ComponentV2
export struct PermissionDialog {
  @Param visible: boolean = false;
  @Param toolName: string = '';
  @Param toolArgs: string = '';
  @Param reason: string = '';
  @Event onAllow?: () => void;
  @Event onDeny?: () => void;

  build() {
    if (this.visible) {
      Stack() {
        // Semi-transparent backdrop
        Column()
          .width('100%')
          .height('100%')
          .backgroundColor('#33000000')
          .onClick(() => {
            // Tap backdrop to deny
            this.onDeny?.();
          })

        // Dialog card
        Column() {
          // Title
          Text('Permission Required')
            .fontSize(18)
            .fontWeight(FontWeight.Bold)
            .fontColor('#E0E0E0')
            .margin({ bottom: 12 })

          // Tool name
          Row() {
            Text('Tool:')
              .fontSize(14)
              .fontColor('#888888')
              .width(60)
            Text(this.toolName)
              .fontSize(14)
              .fontColor('#FFD700')
              .fontWeight(FontWeight.Medium)
          }
          .width('100%')
          .margin({ bottom: 8 })

          // Reason
          Row() {
            Text('Reason:')
              .fontSize(14)
              .fontColor('#888888')
              .width(60)
            Text(this.reason)
              .fontSize(13)
              .fontColor('#FF6B6B')
              .maxLines(3)
              .textOverflow({ overflow: TextOverflow.Ellipsis })
          }
          .width('100%')
          .margin({ bottom: 8 })

          // Args preview
          Text('Arguments:')
            .fontSize(12)
            .fontColor('#666666')
            .alignSelf(ItemAlign.Start)
            .margin({ bottom: 4 })

          Text(this.toolArgs)
            .fontSize(11)
            .fontColor('#AAAAAA')
            .maxLines(5)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .width('100%')
            .padding(8)
            .backgroundColor('#1A1A1A')
            .borderRadius(6)
            .margin({ bottom: 16 })

          // Buttons
          Row() {
            Button('Deny')
              .fontSize(14)
              .fontColor('#FF6B6B')
              .backgroundColor('#2A1A1A')
              .border({ width: 1, color: '#FF6B6B' })
              .borderRadius(8)
              .width('45%')
              .onClick(() => {
                this.onDeny?.();
              })

            Blank()

            Button('Allow')
              .fontSize(14)
              .fontColor('#FFFFFF')
              .backgroundColor('#2E7D32')
              .borderRadius(8)
              .width('45%')
              .onClick(() => {
                this.onAllow?.();
              })
          }
          .width('100%')
        }
        .width('85%')
        .padding(20)
        .backgroundColor('#2A2A2A')
        .borderRadius(16)
        .shadow({ radius: 20, color: '#000000', offsetY: 4 })
      }
      .width('100%')
      .height('100%')
    }
  }
}
