import { RustAgentBridge } from 'hmos_agent_core';

@Entry
@Component
struct Index {
  @State statusText: string = 'Waiting...';
  @State streamOutput: string = '';
  private bridge: RustAgentBridge = new RustAgentBridge();

  aboutToAppear() {
    this.statusText = 'Phase 1 Active';
  }

  build() {
    Scroll() {
      Column({ space: 8 }) {
        Text(this.statusText)
          .fontSize(18)
          .fontWeight(FontWeight.Bold)

        Text('Stream Output:')
          .fontSize(14)

        Text(this.streamOutput)
          .fontSize(12)
          .fontColor('#333')
          .width('100%')
          .constraintSize({ minHeight: 80 })
          .border({ width: 1, color: '#ccc', radius: 8 })
          .padding(8)

        // ── Phase 0 buttons ──
        Button('Init Agent')
          .onClick(() => {
            const ok = this.bridge.init((data: string, eventType: number) => {
              this.streamOutput += `[${eventType}]${data}`;
            });
            this.statusText = ok ? 'Agent Initialized' : 'Init Failed';
          })

        Button('Ping Rust')
          .onClick(() => {
            const resp = this.bridge.call('ping', '{}');
            this.statusText = `Ping: ${resp}`;
          })

        Button('Test Network')
          .onClick(() => {
            const ok = this.bridge.testNetwork();
            this.statusText = ok ? 'Network: OK' : 'Network: FAIL';
          })

        Button('Test Stream')
          .onClick(() => {
            this.streamOutput = '';
            const resp = this.bridge.call('test_stream',
              '{"chunks":10,"interval_ms":100}');
            this.statusText = resp;
          })

        // ── Phase 1 buttons ──
        Button('Search Files')
          .onClick(() => {
            const resp = this.bridge.search('.', 'rust_agent');
            this.statusText = `Search result: ${resp}`;
          })

        Button('Scan Dir')
          .onClick(() => {
            const resp = this.bridge.scanDir('.');
            this.statusText = `Scan result: ${resp}`;
          })
      }
      .width('100%')
      .padding(16)
    }
    .scrollable(ScrollDirection.Vertical)
    .scrollBar(BarState.Off)
    .width('100%')
    .height('100%')
  }
}
