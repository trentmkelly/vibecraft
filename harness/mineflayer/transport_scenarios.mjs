export function createTransportScenarioPlan() {
  return {
    name: 'mineflayer-offline-transport-scenarios',
    mode: 'offline',
    auth: 'offline',
    scenarios: [
      scenario('connection-smoke', ['tcp-accept', 'handshake', 'clean-disconnect']),
      scenario('reconnect-smoke', ['clean-disconnect', 'same-offline-uuid', 'old-connection-removed']),
      scenario('login-cancellation', ['close-after-login-success', 'close-during-registry-sync', 'close-during-first-chunk', 'next-login-not-duplicate-rejected']),
      scenario('wrong-protocol', ['unsupported-version-connect', 'status-response-parity', 'login-disconnect-parity']),
      scenario('connection-refusal', ['before-readiness', 'during-shutdown', 'after-port-close', 'vanilla-compatible-socket-or-disconnect']),
      scenario('socket-cleanup', ['abort-during-handshake', 'abort-during-login-start', 'abort-during-compression', 'abort-during-configuration', 'abort-during-play-entry', 'no-leaked-slots', 'no-pending-keepalive-tasks']),
      scenario('port-reuse', ['repeated-start-stop-same-port', 'one-login-per-cycle', 'no-stale-listener']),
      scenario('parallel-offline-login', ['same-tick-window', 'isolated-handshake-login-packets', 'no-cross-bot-profile-leakage', 'no-compression-leakage', 'no-keepalive-leakage']),
      scenario('transport-framing', ['raw-boundaries-handshake', 'raw-boundaries-login-success', 'raw-boundaries-compression', 'raw-boundaries-configuration-entry', 'official-vs-vibecraft-framing-diff']),
      scenario('half-open-login', ['idle-after-tcp-connect', 'idle-after-handshake', 'idle-after-login-start', 'vanilla-compatible-timeout', 'slot-cleanup', 'later-successful-login']),
      scenario('keepalive', ['multiple-heartbeat-intervals', 'no-false-timeout', 'no-duplicate-response-handling']),
      scenario('malformed-client-behavior', ['unexpected-status-packet', 'unexpected-login-packet', 'unexpected-configuration-packet', 'unexpected-play-packet', 'vanilla-compatible-disconnect-reasons']),
      scenario('compression-threshold', ['disabled-threshold-login', 'low-threshold-login', 'default-threshold-login', 'play-state-reached', 'large-packets-decoded'])
    ]
  }
}

export function summarizeTransportEvidence(evidence, plan = createTransportScenarioPlan()) {
  const scenarios = plan.scenarios.map(entry => {
    const observed = evidence[entry.name] ?? {}
    const missing = entry.required.filter(key => !observed[key])
    return {
      name: entry.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: scenarios.every(result => result.ok),
    scenarios
  }
}

function scenario(name, required) {
  return { name, required }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createTransportScenarioPlan(), null, 2))
}
