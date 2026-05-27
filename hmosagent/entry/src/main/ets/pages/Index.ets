import { ChatView, getPhraseLoader } from 'hmos_agent_core';

@Entry
@Component
struct Index {
  aboutToAppear(): void {
    getPhraseLoader().load(getContext(this));
  }

  build() {
    Column() {
      ChatView()
    }
    .width('100%')
    .height('100%')
  }
}
