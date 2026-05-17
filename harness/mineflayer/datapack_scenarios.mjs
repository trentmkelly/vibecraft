export function createDatapackScenarioPlan() {
  return {
    name: 'mineflayer-offline-datapack-scenarios',
    mode: 'offline',
    auth: 'offline',
    scenarios: [
      {
        name: 'datapack-reload',
        required: [
          'join-before-reload',
          'run-reload-command',
          'registry-tag-resync-observed',
          'bot-survives-where-vanilla-survives',
          'disconnect-reason-recorded-when-vanilla-kicks'
        ]
      },
      {
        name: 'feature-flag-datapack-mismatch',
        required: [
          'changed-enabled-features',
          'changed-datapack-registry-contents',
          'offline-login-attempt',
          'vanilla-compatible-success-or-disconnect',
          'disconnect-component-parity'
        ]
      }
    ]
  }
}

export function summarizeDatapackEvidence(evidence, plan = createDatapackScenarioPlan()) {
  const scenarios = plan.scenarios.map(scenario => {
    const observed = evidence[scenario.name] ?? {}
    const missing = scenario.required.filter(key => !observed[key])
    return {
      name: scenario.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: scenarios.every(result => result.ok),
    scenarios
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createDatapackScenarioPlan(), null, 2))
}
