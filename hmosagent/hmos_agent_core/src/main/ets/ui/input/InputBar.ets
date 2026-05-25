@ComponentV2
export struct InputBar {
  @Param onSend: (text: string) => void = (_text: string) => {};
  @Param onStop: () => void = () => {};
  @Param isStreaming: boolean = false;
  @Local inputText: string = '';

  build() {
    Row() {
      TextInput({ text: $$this.inputText, placeholder: 'Ask anything...' })
        .layoutWeight(1)
        .height(40)
        .fontSize(14)
        .backgroundColor('#f5f5f5')
        .borderRadius(20)
        .padding({ left: 16, right: 16 })
        .onSubmit(() => {
          if (this.inputText.trim()) {
            this.onSend(this.inputText.trim());
            this.inputText = '';
          }
        })

      if (this.isStreaming) {
        Button('■')
          .fontSize(16)
          .fontColor(Color.White)
          .backgroundColor('#e74c3c')
          .borderRadius(20)
          .width(40)
          .height(40)
          .margin({ left: 8 })
          .onClick(() => this.onStop())
      } else {
        Button('↑')
          .fontSize(18)
          .fontColor(Color.White)
          .backgroundColor('#2196F3')
          .borderRadius(20)
          .width(40)
          .height(40)
          .margin({ left: 8 })
          .onClick(() => {
            if (this.inputText.trim()) {
              this.onSend(this.inputText.trim());
              this.inputText = '';
            }
          })
      }
    }
    .width('100%')
    .padding({ left: 12, right: 12, top: 8, bottom: 8 })
    .backgroundColor('#ffffff')
  }
}
