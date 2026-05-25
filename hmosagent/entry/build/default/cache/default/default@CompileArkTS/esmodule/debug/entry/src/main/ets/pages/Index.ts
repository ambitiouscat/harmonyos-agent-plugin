if (!("finalizeConstruction" in ViewPU.prototype)) {
    Reflect.set(ViewPU.prototype, "finalizeConstruction", () => { });
}
interface Index_Params {
    statusText?: string;
    streamOutput?: string;
    bridge?: RustAgentBridge;
}
import { RustAgentBridge } from "@normalized:N&&&hmos_agent_core/Index&1.0.0";
class Index extends ViewPU {
    constructor(parent, params, __localStorage, elmtId = -1, paramsLambda = undefined, extraInfo) {
        super(parent, __localStorage, elmtId, extraInfo);
        if (typeof paramsLambda === "function") {
            this.paramsGenerator_ = paramsLambda;
        }
        this.__statusText = new ObservedPropertySimplePU('Waiting...', this, "statusText");
        this.__streamOutput = new ObservedPropertySimplePU('', this, "streamOutput");
        this.bridge = new RustAgentBridge();
        this.setInitiallyProvidedValue(params);
        this.finalizeConstruction();
    }
    setInitiallyProvidedValue(params: Index_Params) {
        if (params.statusText !== undefined) {
            this.statusText = params.statusText;
        }
        if (params.streamOutput !== undefined) {
            this.streamOutput = params.streamOutput;
        }
        if (params.bridge !== undefined) {
            this.bridge = params.bridge;
        }
    }
    updateStateVars(params: Index_Params) {
    }
    purgeVariableDependenciesOnElmtId(rmElmtId) {
        this.__statusText.purgeDependencyOnElmtId(rmElmtId);
        this.__streamOutput.purgeDependencyOnElmtId(rmElmtId);
    }
    aboutToBeDeleted() {
        this.__statusText.aboutToBeDeleted();
        this.__streamOutput.aboutToBeDeleted();
        SubscriberManager.Get().delete(this.id__());
        this.aboutToBeDeletedInternal();
    }
    private __statusText: ObservedPropertySimplePU<string>;
    get statusText() {
        return this.__statusText.get();
    }
    set statusText(newValue: string) {
        this.__statusText.set(newValue);
    }
    private __streamOutput: ObservedPropertySimplePU<string>;
    get streamOutput() {
        return this.__streamOutput.get();
    }
    set streamOutput(newValue: string) {
        this.__streamOutput.set(newValue);
    }
    private bridge: RustAgentBridge;
    aboutToAppear() {
        this.statusText = 'Phase 0 Gate Active';
    }
    initialRender() {
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Column.create({ space: 12 });
            Column.width('100%');
            Column.padding(16);
        }, Column);
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Text.create(this.statusText);
            Text.fontSize(20);
            Text.fontWeight(FontWeight.Bold);
        }, Text);
        Text.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Text.create('Stream Output:');
            Text.fontSize(16);
        }, Text);
        Text.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Text.create(this.streamOutput);
            Text.fontSize(14);
            Text.fontColor('#333');
            Text.width('90%');
            Text.constraintSize({ minHeight: 100 });
            Text.border({ width: 1, color: '#ccc', radius: 8 });
            Text.padding(12);
        }, Text);
        Text.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Button.createWithLabel('Init Agent');
            Button.onClick(() => {
                const ok = this.bridge.init((data: string, eventType: number) => {
                    this.streamOutput += `[${eventType}]${data}`;
                });
                this.statusText = ok ? 'Agent Initialized' : 'Init Failed';
            });
        }, Button);
        Button.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Button.createWithLabel('Test Network');
            Button.onClick(() => {
                const ok = this.bridge.testNetwork();
                this.statusText = ok ? 'Network: OK' : 'Network: FAIL';
            });
        }, Button);
        Button.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Button.createWithLabel('Ping Rust');
            Button.onClick(async () => {
                const resp = this.bridge.call('ping', '{}');
                this.statusText = `Ping response: ${resp}`;
            });
        }, Button);
        Button.pop();
        this.observeComponentCreation2((elmtId, isInitialRender) => {
            Button.createWithLabel('Test Stream');
            Button.onClick(async () => {
                this.streamOutput = '';
                const resp = this.bridge.call('test_stream', '{"chunks":10,"interval_ms":100}');
                this.statusText = resp;
            });
        }, Button);
        Button.pop();
        Column.pop();
    }
    rerender() {
        this.updateDirtyElements();
    }
    static getEntryName(): string {
        return "Index";
    }
}
registerNamedRoute(() => new Index(undefined, {}), "", { bundleName: "com.example.hmosagent", moduleName: "entry", pagePath: "pages/Index", pageFullPath: "entry/src/main/ets/pages/Index", integratedHsp: "false", moduleType: "followWithHap" });
