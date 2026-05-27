# Tasks: Phase 5 — 极限性能调优与交付

## Gate Criteria

1. `cargo test --lib` 26/26 pass (0 failures)
2. HAR BUILD SUCCESSFUL (0 errors)
3. i3d544 `hap_editor` BUILD SUCCESSFUL
4. SettingsPage 下拉显示 11 个分类（从 JSON 动态加载）
5. 发送对话 → 状态栏显示 `Phrase… (Xs)` 轮播, 1.5s 切换
6. Abort → 状态栏消失

---

## Task 5.1: 趣味短语词库与数据驱动加载 ✅

- [x] 创建 `agent_thinking_phrases.json` (v2.0 categories 格式, 11 分类, completions 内联)
- [x] 创建 `PhraseLoader.ets` 单例 (异步 rawfile 加载, UTF-8 解码, fallback 降级)
- [x] `getCategories()` → 动态分类列表 (SettingsPage 数据源)
- [x] `getSpinnerPhrases(id)` → Map O(1) 查找, fallback → claude-original
- [x] `getRandomCompletion(id)` → category 自包含 completions, fallback → claude-original
- [x] 删除 `SPINNER_TO_COMPLETION` 硬编码映射表
- [x] HAR barrel Index.ets 导出 `getPhraseLoader`
- [x] `Index.ets` aboutToAppear 预加载 PhraseLoader

## Task 5.2: SettingsPage 动态分类与 Thinking Panel 轮播引擎 ✅

- [x] SettingsPage: 删除硬编码 `phrasePresets` 数组
- [x] SettingsPage: `@Local phraseCategories` 从 `getPhraseLoader().getCategories()` 动态填充
- [x] SettingsPage: `thinking_phrase_style` Preferences 持久化 + AppStorage 广播
- [x] ThinkingPanel: `@Local isCollapsed` (V2 @Param 只读规避)
- [x] ThinkingPanel: spinner 显示 `activePhrase… (elapsed)` 含计时
- [x] ThinkingPanel: `@Monitor('isThinking')` 启动/停止 spinner + completion 收尾

## Task 5.3: ChatView 状态栏与 isStreaming 链 ✅

- [x] ChatView: Fisher-Yates shuffle + 1.5s phrase timer + 1s elapsed timer
- [x] ChatView: status bar Row 固定于 InputBar 上方 (streaming 时可见)
- [x] ChatView: `startStatusSpinner()` / `stopStatusSpinner()` 生命周期管理
- [x] ChatView: stream callbacks (eventType 1/2) + error paths 触发 `stopStatusSpinner()`
- [x] MessageBubble: `@Param isStreaming` / `@Param isThinking` 传递
- [x] ChatView → MessageBubble: 仅末位 bubble 传入 `isStreaming=true`

## Task 5.4: 宿主生命周期修复与内存泄漏清理 ✅

- [x] EditorCenterXComponent.ets: 新增 `aboutToDisappear()`
- [x] `aboutToDisappear`: EventHub.off(WEB2CPP), off(key_event), off(CMD_WEB2CPP)
- [x] `aboutToDisappear`: `plugin.threadSafeDataCase(null)` NAPI 回调清理解耦

## Bug Fixes (迭代修复) ✅

- [x] Fix: `PhraseLoader.load()` 从未调用 → `ChatView.aboutToAppear` 预加载
- [x] Fix: `loadPhraseStyle()` async 不 await → 改为同步方法
- [x] Fix: spinner 从未显示 → 状态栏从 MessageBubble 移到 ChatView InputBar 上方
- [x] Fix: i3d544 HAR 版本过旧 → 重新构建 HAR + 部署 (87c913d7)
- [x] Fix: Cargo.toml cdylib 注释归档 (不移除, wasm-pack 需要)

---

## Verification ✅

- [x] HAR BUILD SUCCESSFUL (0 errors)
- [x] HAP 真机部署 + install OK
- [x] SettingsPage: 11 分类动态下拉正常
- [x] 发送对话: 状态栏短语轮播正常 (用户确认)
- [x] `cargo test --lib`: 26/26 pass
- [x] i3d544 `hap_editor` BUILD SUCCESSFUL (49s, 0 errors)

---

7 commits, 12+ files, 3 BUILD SUCCESSFUL.
All tasks complete.
