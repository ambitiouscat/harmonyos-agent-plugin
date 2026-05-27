interface SkillItem {
  name: string;
  description: string;
  category: string;
  params: Array<Record<string, string>>;
}

@ComponentV2
export struct CommandPalette {
  @Param skills: SkillItem[] = [];
  @Param query: string = '';
  @Param visible: boolean = false;
  @Param onSelect: (command: string) => void = (_cmd: string) => {};
  @Param onDismiss: () => void = () => {};
  @Local selectedIdx: number = 0;

  private filteredSkills(): SkillItem[] {
    if (!this.query || this.query.length <= 1) {
      return this.skills;
    }
    const q = this.query.substring(1).toLowerCase();
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

  build() {
    Column() {
      Column() {
        List() {
          ForEach(this.visible ? this.filteredSkills().slice(0, 8) : [],
          (skill: SkillItem, idx: number) => {
          ListItem() {
            Row() {
              Column() {
                Text(skill.name)
                  .fontSize(14)
                  .fontColor(idx === this.selectedIdx ? '#fff' : '#d0d0d0')
                  .fontWeight(idx === this.selectedIdx ? FontWeight.Bold : FontWeight.Normal)
                Text(skill.description)
                  .fontSize(11)
                  .fontColor(idx === this.selectedIdx ? '#a0c4ff' : '#777')
                  .maxLines(1)
                  .textOverflow({ overflow: TextOverflow.Ellipsis })
                  .margin({ top: 2 })
              }
              .alignItems(HorizontalAlign.Start)
              .layoutWeight(1)

              Text(skill.category)
                .fontSize(10)
                .fontColor('#555')
                .padding({ left: 8, right: 8, top: 2, bottom: 2 })
                .borderRadius(4)
                .backgroundColor('rgba(255,255,255,0.06)')
            }
            .width('100%')
            .padding({ left: 12, right: 12, top: 10, bottom: 10 })
            .onClick(() => this.onSelect(skill.name))
            .backgroundColor(idx === this.selectedIdx ?
              'rgba(160,196,255,0.12)' : Color.Transparent)
            .borderRadius(8)
          }
        }, (skill: SkillItem, idx: number) => skill.name)
        }
        .width('100%')
      }
      .width('100%')
      .backgroundColor('#22252b')
      .borderRadius({ topLeft: 12, topRight: 12 })
      .borderWidth({ top: 1, left: 1, right: 1 })
      .borderColor('rgba(255,255,255,0.08)')
      .constraintSize({ maxHeight: 280 })
    }
    .width('100%')
  }
}
