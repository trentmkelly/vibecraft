export function createServerRulesScenarioPlan(options = {}) {
  return {
    name: 'mineflayer-offline-server-rules',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'VibeCraftRules',
    scenarios: [
      scenario('max-player-enforcement', [
        'fill-available-slots',
        'extra-bot-attempt',
        'vanilla-full-server-disconnect'
      ]),
      scenario('whitelist', [
        'offline-allow',
        'offline-deny',
        'runtime-whitelist-reload',
        'enforce-whitelist-toggle'
      ]),
      scenario('mutable-properties', [
        'motd-change',
        'difficulty-change',
        'gamemode-change',
        'view-distance-change',
        'simulation-distance-change',
        'idle-timeout-change',
        'whitelist-change',
        'existing-bot-observes-state',
        'reconnecting-bot-observes-state'
      ]),
      scenario('configuration-reload', [
        'edit-server.properties',
        'vanilla-equivalent-reload',
        'reconnect',
        'restart-required-properties-unchanged',
        'reloadable-properties-applied'
      ]),
      scenario('secure-profile-toggle', [
        'enforce-secure-profile-false-allows-generated-offline-bot',
        'enforce-secure-profile-true-official-comparison',
        'unsigned-mineflayer-client-behavior'
      ])
    ]
  }
}

export function summarizeServerRulesEvidence(evidence, plan = createServerRulesScenarioPlan()) {
  const scenarios = plan.scenarios.map(entry => {
    const observed = evidence[entry.name] ?? {}
    const missing = entry.steps.filter(step => !observed[step])
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

function scenario(name, steps) {
  return { name, steps }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createServerRulesScenarioPlan(), null, 2))
}
