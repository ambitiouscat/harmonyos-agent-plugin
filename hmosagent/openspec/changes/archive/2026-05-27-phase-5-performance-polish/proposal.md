# Proposal: Phase 5 — 极限性能调优与交付

## Why

Phase 0-4 完成了 Rust 核心引擎、ArkUI V2 渲染、Godot 解耦集成和多平台打包验证。但在 2026-05-27 真机测试中发现三项关键缺失：

1. **UX 空白**: Agent 思考等待期间无动态反馈，用户看到静止界面，体验差
2. **内存隐患**: `EditorCenterXComponent` 缺少 `aboutToDisappear`，EventHub 订阅 + NAPI 回调长期持有 UI 实例引用，面板切换导致泄漏
3. **配置僵化**: 趣味短语预设硬编码在代码中，新增分类需改动 SettingsPage

## What

1. **动态趣味短语体系**: PhraseLoader + 11 分类 agent_thinking_phrases.json, Fisher-Yates 轮播, 状态栏计时
2. **GPU-UI 对齐**: 撤销 TS 侧 30fps 双重节流,回归 Rust 16ms/60fps 数据源控制
3. **宿主内存泄漏修复**: aboutToDisappear EventHub 注销 + NAPI threadSafeDataCase(null) 强引用解耦
4. **数据驱动重构**: categories 数组 + 动态下拉 + completions 内联, JSON 作为唯一配置源
5. **i3d544 HAR 同步**: 最新 HAR 部署到 Godot 编辑器宿主

## Scope

- **In**: PhraseLoader.ets, ThinkingPanel REWRITE, SettingsPage phrase picker, ChatView status bar, EditorCenterXComponent 生命周期修复, agent_thinking_phrases.json 重构
- **Out**: 工具集实现 (归于 Phase 6), 多会话管理 (归于 Phase 6)
- **Platforms**: HarmonyOS (HAR + HAP), i3d544 Godot 宿主

## Impact

- **New spec**: thinking-phrases
- **New code**: PhraseLoader.ets, agent_thinking_phrases.json (categories format)
- **Modified code**: ThinkingPanel.ets (rewrite), SettingsPage.ets, ChatView.ets, MessageBubble.ets, Cargo.toml, EditorCenterXComponent.ets (i3d544), HAR barrel Index.ets
- **Risk**: 低 — 纯 ArkTS UI 层变更 + i3d544 生命周期补全，无 Rust FFI 变更
