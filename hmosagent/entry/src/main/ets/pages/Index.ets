import { ChatView } from 'hmos_agent_core';

@Entry
@Component
struct Index {
  build() {
    Column() {
      ChatView()
    }
    .width('100%')
    .height('100%')
  }
}
