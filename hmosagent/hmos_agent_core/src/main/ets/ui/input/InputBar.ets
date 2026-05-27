interface SkillItem {
  name: string;
  description: string;
  category: string;
  params: Array<Record<string, string>>;
}

@ComponentV2
export struct InputBar {
  @Param onSend: (text: string) => void = (_text: string) => {};
  @Param onStop: () => void = () => {};
  @Param isStreaming: boolean = false;
  @Param skills: SkillItem[] = [];
  @Param themeColor: string = 'dark';
  @Local inputText: string = '';
  @Local showPalette: boolean = false;

  private onTextChange(value: string): void {
    this.inputText = value;
    if (value.startsWith('/') && this.skills.length > 0) {
      this.showPalette = true;
    } else {
      this.showPalette = false;
    }
  }

  private filteredSkills(): SkillItem[] {
    const q = this.inputText.substring(1).toLowerCase();
    if (!q) return this.skills.slice(0, 6);
    const result: SkillItem[] = [];
    for (let i = 0; i < this.skills.length; i++) {
      const s = this.skills[i];
      if (s.name.toLowerCase().indexOf(q) >= 0 ||
          s.description.toLowerCase().indexOf(q) >= 0) {
        result.push(s);
      }
    }
    return result.slice(0, 6);
  }

  private handleSelectCommand(cmd: string): void {
    this.inputText = cmd + ' ';
    this.showPalette = false;
  }

  build() {
    Column() {
      // Command suggestions (above input)
      if (this.showPalette) {
        Column() {
          Text('Commands')
            .fontSize(11)
            .fontColor(this.themeColor === 'dark' ? '#666' : '#888')
            .padding({ left: 12, top: 8, bottom: 4 })
          ForEach(this.filteredSkills(), (skill: SkillItem) => {
            Row() {
              Column() {
                Text(skill.name)
                  .fontSize(14)
                  .fontColor(this.themeColor === 'dark' ? '#d0d0d0' : '#222')
                Text(skill.description)
                  .fontSize(11)
                  .fontColor(this.themeColor === 'dark' ? '#777' : '#666')
                  .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
              }
              .alignItems(HorizontalAlign.Start).layoutWeight(1)
              Text(skill.category)
                .fontSize(10)
                .fontColor(this.themeColor === 'dark' ? '#555' : '#888')
                .padding({ left: 6, right: 6, top: 2, bottom: 2 })
                .borderRadius(4)
                .backgroundColor(this.themeColor === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)')
            }
            .width('100%').padding({ left: 12, right: 12, top: 6, bottom: 6 })
            .onClick(() => this.handleSelectCommand(skill.name))
          }, (skill: SkillItem): string => skill.name)
        }
        .width('100%')
        .backgroundColor(this.themeColor === 'dark' ? '#22252b' : '#f0f0f0')
        .borderRadius({ topLeft: 12, topRight: 12 })
        .borderWidth({ top: 1, left: 1, right: 1 })
        .borderColor(this.themeColor === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.08)')
        .constraintSize({ maxHeight: 200 })
      }

      // Input row
      Row() {
        TextInput({ text: $$this.inputText, placeholder: 'Ask anything... (/ for commands)' })
          .layoutWeight(1)
          .height(40)
          .fontSize(14)
          .backgroundColor(this.themeColor === 'dark' ? '#2a2d33' : '#f5f5f5')
          .fontColor(this.themeColor === 'dark' ? '#e0e0e0' : '#1a1a1a')
          .borderRadius(20)
          .padding({ left: 16, right: 16 })
          .onChange((value: string) => this.onTextChange(value))
          .onSubmit(() => {
            if (this.inputText.trim()) {
              this.onSend(this.inputText.trim());
              this.inputText = '';
              this.showPalette = false;
            }
          })

        if (this.isStreaming) {
          Button() {
            Image($r('app.media.icon_stop'))
              .width(18).height(18)
              .fillColor(Color.White)
          }
            .backgroundColor('#e74c3c')
            .borderRadius(20)
            .width(40).height(40)
            .margin({ left: 8 })
            .onClick(() => this.onStop())
        } else {
          Button() {
            Image($r('app.media.icon_send'))
              .width(20).height(20)
              .fillColor(Color.White)
          }
            .backgroundColor('#2196F3')
            .borderRadius(20)
            .width(40).height(40)
            .margin({ left: 8 })
            .onClick(() => {
              if (this.inputText.trim()) {
                this.onSend(this.inputText.trim());
                this.inputText = '';
                this.showPalette = false;
              }
            })
        }
      }
      .width('100%')
      .padding({ left: 12, right: 12, top: 4, bottom: 8 })
      .backgroundColor('#ffffff')
    }
    .width('100%')
  }
}
