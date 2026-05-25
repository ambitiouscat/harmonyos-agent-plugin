@ObservedV2
export class ArkContentPart {
  public type: 'text' | 'reasoning' | 'tool_call' | 'tool_result';

  @Trace public text: string = '';
  @Trace public collapsed: boolean = false;
  @Trace public toolId: string = '';
  @Trace public name: string = '';
  @Trace public isError: boolean = false;

  constructor(type: 'text' | 'reasoning' | 'tool_call' | 'tool_result') {
    this.type = type;
  }
}
