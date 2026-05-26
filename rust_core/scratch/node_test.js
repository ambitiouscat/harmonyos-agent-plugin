const { node_agent_init, node_agent_call } = require('../pkg/node/agent_core.node');

const ok = node_agent_init(null);
console.log(`init: ${ok}`);

const res = node_agent_call("ping", "{}");
console.log(`ping: ${res}`);

const parsed = JSON.parse(res);
if (parsed.status === 'ok' && parsed.message === 'pong') {
  console.log('[PASS] Ping test passed');
  process.exit(0);
} else {
  console.log('[FAIL] Ping test failed');
  process.exit(1);
}
