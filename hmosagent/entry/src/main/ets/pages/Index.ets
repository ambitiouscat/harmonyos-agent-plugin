import { RustAgentBridge } from 'hmos_agent_core';

@Entry
@Component
struct Index {
  @State statusText: string = 'Waiting...';
  @State streamOutput: string = '';
  private bridge: RustAgentBridge = new RustAgentBridge();

  aboutToAppear() {
    this.statusText = 'Phase 0 Gate Active';
  }

  build() {
    Column({ space: 12 }) {
      Text(this.statusText)
        .fontSize(20)
        .fontWeight(FontWeight.Bold)

      Text('Stream Output:')
        .fontSize(16)

      Text(this.streamOutput)
        .fontSize(14)
        .fontColor('#333')
        .width('90%')
        .constraintSize({ minHeight: 100 })
        .border({ width: 1, color: '#ccc', radius: 8 })
        .padding(12)

      Button('Init Agent')
        .onClick(() => {
          const ok = this.bridge.init((data: string, eventType: number) => {
            this.streamOutput += `[${eventType}]${data}`;
          });
          this.statusText = ok ? 'Agent Initialized' : 'Init Failed';
        })

      Button('Test Network')
        .onClick(() => {
          const ok = this.bridge.testNetwork();
          this.statusText = ok ? 'Network: OK' : 'Network: FAIL';
        })

      Button('Ping Rust')
        .onClick(async () => {
          const resp = this.bridge.call('ping', '{}');
          this.statusText = `Ping response: ${resp}`;
        })

      Button('Test Stream')
        .onClick(async () => {
          this.streamOutput = '';
          const resp = this.bridge.call('test_stream',
            '{"chunks":10,"interval_ms":100}');
          this.statusText = resp;
        })
    }
    .width('100%')
    .padding(16)
  }
}
