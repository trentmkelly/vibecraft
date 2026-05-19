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

export function buildDatapackReloadEvidence({ beforeReload, afterReload, official }) {
  const vanillaSurvives = official.afterReload?.joined === true
  const rustCraftSurvives = afterReload?.joined === true
  return {
    'join-before-reload': beforeReload?.joined === true,
    'run-reload-command': afterReload?.reloadCommandSent === true,
    'registry-tag-resync-observed': resyncObserved(beforeReload, afterReload),
    'bot-survives-where-vanilla-survives': vanillaSurvives ? rustCraftSurvives : true,
    'disconnect-reason-recorded-when-vanilla-kicks': vanillaSurvives || typeof afterReload?.disconnectReason === 'string',
    disconnectReason: afterReload?.disconnectReason ?? null
  }
}

export function buildFeatureFlagDatapackMismatchEvidence({ attempt, official }) {
  const bothSucceeded = attempt.joined === true && official.joined === true
  const bothDisconnected = attempt.joined === false && official.joined === false
  return {
    'changed-enabled-features': attempt.changedEnabledFeatures === true,
    'changed-datapack-registry-contents': attempt.changedDatapackRegistryContents === true,
    'offline-login-attempt': attempt.offlineLoginAttempted === true,
    'vanilla-compatible-success-or-disconnect': bothSucceeded || bothDisconnected,
    'disconnect-component-parity': bothSucceeded || sameDisconnectComponent(attempt.disconnectReason, official.disconnectReason)
  }
}

function resyncObserved(beforeReload, afterReload) {
  return beforeReload?.registryHash !== afterReload?.registryHash ||
    beforeReload?.tagHash !== afterReload?.tagHash ||
    afterReload?.registryTagResyncObserved === true
}

function sameDisconnectComponent(left, right) {
  if (typeof left !== 'string' || typeof right !== 'string') return false
  return left === right
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createDatapackScenarioPlan(), null, 2))
}
