export function createTickStabilityPlan(options = {}) {
  return {
    name: 'mineflayer-offline-tick-stability',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'VibeCraftTick',
    transitions: [
      'pause-when-empty',
      'autosave',
      'save-off',
      'save-on',
      'save-all'
    ],
    requiredEvidence: [
      'bot-connected-through-pause-transition',
      'autosave-boundary-observed',
      'save-off-command-accepted',
      'save-on-command-accepted',
      'save-all-command-accepted',
      'keepalive-request-response-after-each-transition',
      'visible-state-does-not-stall'
    ],
    commands: ['/save-off', '/save-on', '/save-all']
  }
}

export function summarizeTickStabilityEvidence(evidence, plan = createTickStabilityPlan()) {
  const missing = plan.requiredEvidence.filter(key => !evidence[key])
  const transitionResults = Object.fromEntries(plan.transitions.map(transition => [
    transition,
    Boolean(evidence.transitions?.[transition])
  ]))
  const missingTransitions = Object.entries(transitionResults)
    .filter(([, ok]) => !ok)
    .map(([name]) => name)

  return {
    ok: missing.length === 0 && missingTransitions.length === 0,
    missing,
    transitions: transitionResults,
    missingTransitions
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createTickStabilityPlan(), null, 2))
}
