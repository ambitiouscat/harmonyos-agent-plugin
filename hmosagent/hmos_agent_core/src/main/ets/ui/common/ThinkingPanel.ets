import { getPhraseLoader } from './PhraseLoader';

@ComponentV2
export struct ThinkingPanel {
  @Param @Require content: string = '';
  @Param collapsed: boolean = true;
  @Param isThinking: boolean = false;
  @Param isStreaming: boolean = false;
  @Local activePhrase: string = 'Thinking...';
  @Local isCollapsed: boolean = true;
  @Local showCompletion: boolean = false;
  @Local completionText: string = '';
  @Local fadeOut: boolean = false;
  private phraseQueue: string[] = [];
  private queueIndex: number = 0;
  private timerId: number = -1;
  private phraseStyle: string = 'claude-original';
  private loader = getPhraseLoader();

  aboutToAppear(): void {
    this.isCollapsed = this.collapsed;
    this.startSpinner();
  }

  aboutToDisappear(): void {
    this.stopTimer();
  }

  @Monitor('isThinking')
  onThinkingChange(): void {
    if (this.isThinking) {
      this.showCompletion = false;
      this.fadeOut = false;
      this.startSpinner();
    } else {
      this.stopTimer();
      this.showCompletionWord();
    }
  }

  @Monitor('isStreaming')
  onStreamingChange(): void {
    if (!this.isStreaming && !this.isThinking) {
      this.stopTimer();
      this.showCompletionWord();
    }
  }

  startSpinner(): void {
    this.stopTimer();
    this.loadPhraseStyle();
    this.buildQueue();
    if (this.phraseQueue.length === 0) {
      this.activePhrase = 'Thinking...';
      return;
    }
    this.activePhrase = this.phraseQueue[0];
    this.queueIndex = 0;
    this.timerId = setInterval(() => {
      this.queueIndex++;
      if (this.queueIndex >= this.phraseQueue.length) {
        this.buildQueue();
        this.queueIndex = 0;
      }
      this.activePhrase = this.phraseQueue[this.queueIndex];
    }, 1500);
  }

  stopTimer(): void {
    if (this.timerId !== -1) {
      clearInterval(this.timerId);
      this.timerId = -1;
    }
  }

  buildQueue(): void {
    const words = this.loader.getSpinnerPhrases(this.phraseStyle);
    if (words.length === 0) {
      this.phraseQueue = ['Thinking...'];
      return;
    }
    this.phraseQueue = this.fisherYatesShuffle([...words]);
    if (this.phraseQueue.length > 1 && this.phraseQueue[0] === this.activePhrase) {
      const tmp = this.phraseQueue[0];
      this.phraseQueue[0] = this.phraseQueue[1];
      this.phraseQueue[1] = tmp;
    }
  }

  fisherYatesShuffle(arr: string[]): string[] {
    for (let i: number = arr.length - 1; i > 0; i--) {
      const j: number = Math.floor(Math.random() * (i + 1));
      const tmp: string = arr[i];
      arr[i] = arr[j];
      arr[j] = tmp;
    }
    return arr;
  }

  showCompletionWord(): void {
    if (this.showCompletion) return;
    this.showCompletion = true;
    this.completionText = this.loader.getRandomCompletion(this.phraseStyle);
    this.fadeOut = false;
    setTimeout(() => {
      this.fadeOut = true;
    }, 1000);
  }

  async loadPhraseStyle(): Promise<void> {
    try {
      const val = AppStorage.get<string>('thinking_phrase_style');
      if (val) {
        this.phraseStyle = val;
      }
    } catch (_e) {
      this.phraseStyle = 'claude-original';
    }
  }

  build() {
    Column() {
      Row() {
        Rect()
          .width(3)
          .height('100%')
          .fill('#888')

        Column() {
          Row() {
            if (this.showCompletion) {
              Text(this.completionText)
                .fontSize(13)
                .fontColor('#4CAF50')
                .fontWeight(FontWeight.Medium)
                .opacity(this.fadeOut ? 0 : 1)
                .animation({ duration: 500, curve: Curve.EaseOut })
            } else if (this.isThinking && !this.content) {
              Text(this.activePhrase)
                .fontSize(13)
                .fontColor('#888')
                .fontStyle(FontStyle.Italic)
            } else {
              Text('Thinking')
                .fontSize(12)
                .fontColor('#666')
            }
            Blank()
            Text(this.isCollapsed ? '▶' : '▼')
              .fontSize(12)
              .fontColor('#999')
          }
          .width('100%')
          .padding({ bottom: 4 })

          if (!this.isCollapsed && this.content) {
            Text(this.content)
              .fontSize(13)
              .fontColor('#555')
              .lineHeight(18)
          }
        }
        .layoutWeight(1)
        .padding({ left: 8 })
      }
      .width('100%')
      .alignItems(VerticalAlign.Top)
    }
    .padding(12)
    .backgroundColor('#f0f0f020')
    .borderRadius(8)
    .onClick(() => {
      this.isCollapsed = !this.isCollapsed;
    })
  }
}
