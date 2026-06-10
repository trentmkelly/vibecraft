import { runObservedOfflineLogin } from './login_session.mjs'
import { stopServer, waitForPort, writeOfflineServerFiles } from './runner.mjs'
import { forceReconnect, waitForSpawn } from './bot_actions.mjs'

export function createLifecycleScenarioPlan(options = {}) {
  return {
    name: 'mineflayer-offline-lifecycle',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'VibeCraftLifecycle',
    scenarios: [
      scenario('shutdown-restart', [
        'join',
        'mutate-visible-state',
        'clean-stop',
        'restart',
        'reconnect',
        'persisted-position',
        'persisted-inventory',
        'persisted-stats'
      ]),
      scenario('crash-after-login-recovery', [
        'join',
        'first-save-boundary',
        'terminate-process',
        'restart',
        'reconnect',
        'no-corrupted-player-files',
        'no-corrupted-world-files'
      ]),
      scenario('startup-race', [
        'connect-during-bootstrap',
        'held-until-ready-or-vanilla-rejection',
        'official-timing-envelope'
      ]),
      scenario('first-login-bootstrap', [
        'eula-only-start',
        'generated-config',
        'generated-world',
        'generated-player-artifacts',
        'vanilla-order-before-play'
      ]),
      scenario('failed-start-cleanup', [
        'missing-eula-refusal',
        'bad-world-metadata-refusal',
        'invalid-properties-refusal',
        'no-partial-player-files',
        'no-stale-bot-session'
      ]),
      scenario('lifecycle-artifacts', [
        'startup-logs',
        'readiness-signal',
        'first-accepted-login-tick',
        'shutdown-reason',
        'post-exit-file-flush'
      ]),
      scenario('interrupted-bootstrap', [
        'kill-mid-login',
        'restart',
        'no-half-created-player',
        'no-stale-socket',
        'no-stale-world-lock'
      ]),
      scenario('eula-refusal', [
        'login-before-eula',
        'vanilla-compatible-refusal',
        'accept-eula',
        'same-bot-can-join',
        'no-stale-state'
      ])
    ],
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export function summarizeLifecycleEvidence(evidence, plan = createLifecycleScenarioPlan()) {
  const scenarioResults = plan.scenarios.map(entry => {
    const observed = evidence[entry.name] ?? {}
    const missing = entry.steps.filter(step => !Boolean(observed[step]))
    return {
      name: entry.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: scenarioResults.every(result => result.ok),
    scenarios: scenarioResults
  }
}

export async function runShutdownRestartScenario(options = {}) {
  const session = await runObservedOfflineLogin({
    ...options,
    username: options.username ?? 'VibeCraftLifecycle',
    properties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      ...(options.properties ?? {})
    },
    keepAlive: true
  })
  if (session.error) throw session.error

  try {
    await waitForSpawn(session, { timeoutMs: options.timeoutMs })
    session.timeline.push({ name: 'action', at: Date.now(), summary: ['mutate-visible-state', {}] })
    await stopServer(session.server.child)
    session.timeline.push({ name: 'action', at: Date.now(), summary: ['clean-stop', {}] })
    await writeOfflineServerFiles(session.root, { ...options, port: session.endpoint.port })
    session.server = (options.startServer ?? (() => null))({
      ...options,
      root: session.root,
      port: session.endpoint.port
    }) ?? session.server
    await (options.waitForReady ?? waitForPort)(session.endpoint.port, session.endpoint.host, options.timeoutMs)
    await forceReconnect(session, options)
    return session
  } finally {
    await session.cleanup()
  }
}

function scenario(name, steps) {
  return { name, steps }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const plan = createLifecycleScenarioPlan()
  console.log(JSON.stringify(plan, null, 2))
}
