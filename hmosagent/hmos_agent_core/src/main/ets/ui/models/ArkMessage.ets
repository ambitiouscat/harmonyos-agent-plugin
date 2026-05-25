import { ArkContentPart } from './ArkContentPart';

@ObservedV2
export class ArkMessage {
  public id: string;
  public role: 'system' | 'user' | 'assistant' | 'tool';

  @Trace public parts: ArkContentPart[] = [];
  @Trace public model: string = '';
  public createdAt: string;

  constructor(role: 'system' | 'user' | 'assistant' | 'tool', id?: string) {
    this.id = id || `msg_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
    this.role = role;
    this.createdAt = new Date().toISOString();
  }
}
