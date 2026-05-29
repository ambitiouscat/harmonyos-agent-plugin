import { KeyCode } from '@kit.InputKit';

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
  @Local selectedIdx: number = 0;
  private scroller: Scroller = new Scroller();
  private longPressActive: boolean = false;
  private longPressTimer: number = 0;
  private lastRepeatTime: number = 0;

  @Monitor('selectedIdx')
  onSelectedIdxChange(): void {
    if (this.showPalette) {
      this.scroller.scrollToIndex(this.selectedIdx, true, ScrollAlign.CENTER);
    }
  }

  private onTextChange(value: string): void {
    this.inputText = value;
    if (value.startsWith('/') && this.skills.length > 0) {
      this.showPalette = true;
      this.selectedIdx = 0;
    } else {
      this.showPalette = false;
      this.selectedIdx = 0;
    }
  }

  private normalizeName(name: string): string {
    if (!name || name.length === 0) return name;
    return name.startsWith('/') ? name : '/' + name;
  }

  private filteredSkills(): SkillItem[] {
    const q = this.inputText.substring(1).toLowerCase();
    if (!q) return this.skills;
    const result: SkillItem[] = [];
    for (let i = 0; i < this.skills.length; i++) {
      const s = this.skills[i];
      if (s.name.toLowerCase().indexOf(q) >= 0 ||
          s.description.toLowerCase().indexOf(q) >= 0) {
        result.push(s);
      }
    }
    return result;
  }

  private handleSelectCommand(cmd: string): void {
    this.inputText = this.normalizeName(cmd) + ' ';
    this.showPalette = false;
    this.selectedIdx = 0;
    setTimeout(() => {
      focusControl.requestFocus('chat_input_field');
    }, 100);
  }

  build() {
    Stack({ alignContent: Alignment.Bottom }) {
      // Input row (base layer)
      Row() {
        TextInput({ text: $$this.inputText, placeholder: 'Ask anything... (/ for commands)' })
          .id('chat_input_field')
          .layoutWeight(1)
          .height(40)
          .fontSize(14)
          .backgroundColor(this.themeColor === 'dark' ? '#2a2d33' : '#f5f5f5')
          .fontColor(this.themeColor === 'dark' ? '#e0e0e0' : '#1a1a1a')
          .borderRadius(20)
          .padding({ left: 16, right: 16 })
          .onChange((value: string) => this.onTextChange(value))
          .onSubmit(() => {
            if (this.showPalette) {
              const filtered = this.filteredSkills();
              if (filtered.length > 0) {
                this.handleSelectCommand(filtered[this.selectedIdx].name);
                return;
              }
            }
            if (this.isStreaming) return;
            if (this.inputText.trim()) {
              this.onSend(this.inputText.trim());
              this.inputText = '';
              this.showPalette = false;
              this.selectedIdx = 0;
            }
          })
          .onKeyPreIme((event: KeyEvent) => {
            if (!this.showPalette) return false;
            const filtered = this.filteredSkills();
            if (filtered.length === 0) return false;

            const isUp = event.type === KeyType.Up;
            const isDown = event.type === KeyType.Down;
            const isArrow = event.keyCode === KeyCode.KEYCODE_DPAD_DOWN ||
                            event.keyCode === KeyCode.KEYCODE_DPAD_UP;

            if (isArrow) {
              if (isDown) {
                // Start timer on first key-down
                if (this.longPressTimer === 0) {
                  this.longPressTimer = Date.now();
                }
                const now = Date.now();
                const holdDuration = now - this.longPressTimer;

                // Switch to long-press mode after 400ms hold
                if (!this.longPressActive && holdDuration > 400) {
                  this.longPressActive = true;
                  this.lastRepeatTime = now;
                }

                if (!this.longPressActive) {
                  // Short press (< 400ms hold): only first key-down moves
                  if (holdDuration < 50) {
                    if (event.keyCode === KeyCode.KEYCODE_DPAD_DOWN) {
                      this.selectedIdx = (this.selectedIdx + 1) % filtered.length;
                    } else {
                      this.selectedIdx = (this.selectedIdx - 1 + filtered.length) % filtered.length;
                    }
                  }
                } else {
                  // Long press: rapid repeat every 80ms
                  if (now - this.lastRepeatTime > 80) {
                    if (event.keyCode === KeyCode.KEYCODE_DPAD_DOWN) {
                      this.selectedIdx = (this.selectedIdx + 1) % filtered.length;
                    } else {
                      this.selectedIdx = (this.selectedIdx - 1 + filtered.length) % filtered.length;
                    }
                    this.lastRepeatTime = now;
                  }
                }
              } else if (isUp) {
                // Reset on release
                this.longPressActive = false;
                this.longPressTimer = 0;
                this.lastRepeatTime = 0;
              }
              return true;
            }

            if (event.keyCode === KeyCode.KEYCODE_TAB) {
              if (isUp) {
                this.handleSelectCommand(filtered[this.selectedIdx].name);
              }
              return true;
            }
            return false;
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
                this.selectedIdx = 0;
              }
            })
        }
      }
      .width('100%')
      .padding({ left: 12, right: 12, top: 4, bottom: 8 })
      .backgroundColor('rgba(255,255,255,0)') // transparent to show palette through

      // Command palette (overlay above input)
      if (this.showPalette) {
        Column() {
          Text('Commands')
            .fontSize(11)
            .fontColor(this.themeColor === 'dark' ? '#666' : '#888')
            .padding({ left: 12, top: 8, bottom: 4 })
          List({ scroller: this.scroller }) {
            ForEach(this.filteredSkills(), (skill: SkillItem, idx: number) => {
              ListItem() {
                Row() {
                  Column() {
                    Text(skill.name)
                      .fontSize(14)
                      .fontColor(idx === this.selectedIdx ? '#fff' :
                        (this.themeColor === 'dark' ? '#d0d0d0' : '#222'))
                      .fontWeight(idx === this.selectedIdx ? FontWeight.Bold : FontWeight.Normal)
                    Text(skill.description)
                      .fontSize(11)
                      .fontColor(idx === this.selectedIdx ? '#a0c4ff' :
                        (this.themeColor === 'dark' ? '#777' : '#666'))
                      .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
                  }
                  .alignItems(HorizontalAlign.Start).layoutWeight(1)
                  Text(skill.category)
                    .fontSize(10)
                    .fontColor(idx === this.selectedIdx ? '#a0c4ff' :
                      (this.themeColor === 'dark' ? '#555' : '#888'))
                    .padding({ left: 6, right: 6, top: 2, bottom: 2 })
                    .borderRadius(4)
                    .backgroundColor(idx === this.selectedIdx ?
                      'rgba(160,196,255,0.15)' :
                      (this.themeColor === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)'))
                }
                .width('100%').padding({ left: 12, right: 12, top: 6, bottom: 6 })
                .backgroundColor(idx === this.selectedIdx ?
                  'rgba(160,196,255,0.1)' : Color.Transparent)
                .borderRadius(8)
                .onClick(() => this.handleSelectCommand(skill.name))
              }
            }, (skill: SkillItem): string => skill.name)
          }
          .width('100%')
          .scrollBar(BarState.Auto)
          .constraintSize({ maxHeight: 200 })
        }
        .width('100%')
        .backgroundColor(this.themeColor === 'dark' ? '#22252b' : '#f0f0f0')
        .borderRadius({ topLeft: 12, topRight: 12 })
        .borderWidth({ top: 1, left: 1, right: 1 })
        .borderColor(this.themeColor === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.08)')
        .constraintSize({ maxHeight: 240 })
        .clip(true)
        .margin({ bottom: 48 }) // sits above the input row height
      }
    }
    .width('100%')
  }
}
