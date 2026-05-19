export const MULTIPLAYER_PRESENTATION_SCENARIOS = {
  lifecycle: ['join-broadcast', 'tab-list-add-remove', 'quit-message', 'kick-reason', 'respawn', 'transfer-rejection-acceptance', 'reconnect-persistence'],
  loginQueueOrdering: ['controlled-gate-release', 'accepted-player-order', 'tab-list-order', 'join-message-order'],
  simultaneousOfflineLogin: ['multiple-generated-profiles', 'join-order', 'spawn-collision-handling', 'chat-visibility', 'player-list-latency', 'disconnect-cleanup'],
  loginStorm: ['many-generated-profiles', 'randomized-ports-worlds', 'accepted-rejected-counts', 'tick-latency', 'cleanup'],
  joinQuitBroadcast: ['first-login-message', 'returning-login-message', 'duplicate-login-message', 'kick-message', 'timeout-message', 'crash-disconnect-message', 'vanilla-comparison'],
  sameTickLoginLogout: ['rapid-connect-disconnect', 'tab-list-cleanup', 'entity-id-cleanup', 'keepalive-cleanup', 'playerdata-isolation'],
  duplicateSessionCleanup: ['reconnect-before-close', 'entity-removal', 'tab-list-replacement', 'playerdata-ownership', 'kicked-message-parity'],
  rapidReconnect: ['repeat-same-name', 'stale-entity-cleanup', 'tab-list-cleanup', 'keepalive-task-cleanup', 'player-file-cleanup'],
  mixedProfileMultiplayer: ['op-profile', 'non-op-profile', 'whitelisted-profile', 'banned-profile', 'duplicate-profile', 'accepted-ordering', 'rejection-reasons', 'broadcasts', 'tab-list-state'],
  reconnectAfterKick: ['kick', 'ban', 'unban', 'reconnect', 'stale-session-cleanup', 'kicked-message-parity'],
  presentation: ['action-bar', 'title-subtitle-times', 'bossbar-add-update-remove', 'tab-list-header-footer', 'death-message-formatting'],
  tabListMutation: ['latency-change', 'display-name-change', 'game-mode-change', 'listed-flag-change', 'hat-visibility-change', 'list-order-change', 'packet-event-order'],
  profilePropertyTabList: ['empty-profile-properties', 'synthetic-profile-properties', 'vanilla-compatible-serialization']
}

export function createMultiplayerPresentationPlan(kind, options = {}) {
  const steps = MULTIPLAYER_PRESENTATION_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown multiplayer/presentation scenario ${kind}`)
  return {
    name: `mineflayer-multiplayer-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'MultiBot',
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runMultiplayerPresentationScenario(kind, options = {}) {
  const plan = createMultiplayerPresentationPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? multiplayerProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing multiplayer/presentation evidence for ${step}`)
    recordMultiplayerPresentation(evidence, step, result.details?.[step] ?? {})
  }
  return { plan, evidence, summary: summarizeMultiplayerPresentation(evidence, plan) }
}

export function summarizeMultiplayerPresentation(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'multiplayer_presentation')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function recordMultiplayerPresentation(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'multiplayer_presentation', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function multiplayerProbe() {
  throw new Error('multiplayerProbe requires a scenario-specific server fixture')
}
