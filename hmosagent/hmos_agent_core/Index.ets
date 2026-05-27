export { RustAgentBridge } from './src/main/ets/RustAgentBridge';
export { ChatView } from './src/main/ets/ui/chat/ChatView';
export { SettingsPage } from './src/main/ets/pages/SettingsPage';
export { ArkMessage } from './src/main/ets/ui/models/ArkMessage';
export { ArkContentPart } from './src/main/ets/ui/models/ArkContentPart';
export { getPhraseLoader } from './src/main/ets/ui/common/PhraseLoader';

export const CMD_WEB2CPP: string = "agent:web2cpp";
export const EVENT_CPP2WEB: string = "agent:cpp2web";
