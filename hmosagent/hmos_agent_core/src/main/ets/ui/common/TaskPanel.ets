/**
 * TaskPanel — right-side slide-out panel showing task progress.
 *
 * Displays all tasks from the Rust TaskStore with status icons:
 *   ○ pending    ◌ in_progress    ✓ completed
 * blocked_by dependencies are shown as indented subtasks.
 */
interface TaskItem {
  id: string;
  content: string;
  status: string;
  blocked_by: string[];
  owner?: string;
}

@ComponentV2
export struct TaskPanel {
  @Param isOpen: boolean = false;
  @Param tasks: TaskItem[] = [];
  @Event onClose: () => void = () => {};

  build() {
    if (this.isOpen) {
      Stack() {
        // Backdrop
        Column()
          .width('100%')
          .height('100%')
          .backgroundColor('rgba(0,0,0,0.3)')
          .onClick(() => this.onClose())

        // Panel slides from right
        Column() {
          // Header
          Row() {
            Text('Tasks')
              .fontSize(18)
              .fontWeight(FontWeight.Bold)
              .fontColor('#E0E0E0')
            Blank()
            // Stats
            Text(this.taskStats())
              .fontSize(11)
              .fontColor('#888888')
            Blank()
            Image($r('app.media.icon_close'))
              .width(20).height(20)
              .fillColor('#888888')
              .onClick(() => this.onClose())
          }
          .width('100%')
          .padding(16)
          .borderWidth({ bottom: 1 })
          .borderColor('rgba(255,255,255,0.06)')

          // Task list
          if (this.tasks.length === 0) {
            Column() {
              Text('No tracked tasks')
                .fontSize(13)
                .fontColor('#666666')
                .margin({ top: 40 })
            }
            .width('100%')
            .layoutWeight(1)
          } else {
            List() {
              ForEach(this.tasks, (task: TaskItem) => {
                ListItem() {
                  Row() {
                    // Status icon
                    Text(this.statusIcon(task.status))
                      .fontSize(16)
                      .fontColor(this.statusColor(task.status))
                      .width(24)
                      .textAlign(TextAlign.Center)

                    // Content
                    Column() {
                      Text(task.content)
                        .fontSize(13)
                        .fontColor('#D0D0D0')
                        .maxLines(2)
                        .textOverflow({ overflow: TextOverflow.Ellipsis })
                        .decoration({
                          type: task.status === 'completed' ? TextDecorationType.LineThrough : TextDecorationType.None
                        })

                      // Show blocked_by dependencies
                      if (task.blocked_by.length > 0) {
                        Row() {
                          Text('⛓')
                            .fontSize(10)
                            .margin({ right: 4 })
                          Text('Waiting: ' + task.blocked_by.join(', '))
                            .fontSize(10)
                            .fontColor('#888888')
                            .maxLines(1)
                            .textOverflow({ overflow: TextOverflow.Ellipsis })
                        }
                        .margin({ top: 2 })
                      }
                    }
                    .layoutWeight(1)
                    .margin({ left: 8 })
                  }
                  .width('100%')
                  .padding({ left: 16, right: 16, top: 10, bottom: 10 })
                }
              }, (task: TaskItem): string => task.id)
            }
            .width('100%')
            .layoutWeight(1)
            .scrollBar(BarState.Off)
          }
        }
        .width('78%')
        .height('100%')
        .backgroundColor('#2A2A2A')
        .borderRadius({ topLeft: 16, bottomLeft: 16 })
        .alignSelf(ItemAlign.End)
        .transition(TransitionEffect.asymmetric(
          TransitionEffect.translate({ x: 800 })
            .animation({ duration: 220, curve: Curve.FastOutSlowIn }),
          TransitionEffect.translate({ x: 800 })
            .animation({ duration: 180, curve: Curve.FastOutLinearIn })
        ))
      }
      .width('100%')
      .height('100%')
    }
  }

  private statusIcon(status: string): string {
    switch (status) {
      case 'pending': return '○';
      case 'in_progress': return '◌';
      case 'completed': return '✓';
      default: return '○';
    }
  }

  private statusColor(status: string): string {
    switch (status) {
      case 'pending': return '#888888';
      case 'in_progress': return '#FFD700';
      case 'completed': return '#4CAF50';
      default: return '#888888';
    }
  }

  private taskStats(): string {
    const total = this.tasks.length;
    if (total === 0) return 'empty';
    let done = 0;
    for (let i = 0; i < this.tasks.length; i++) {
      if (this.tasks[i].status === 'completed') done++;
    }
    return `${done}/${total} done`;
  }
}
