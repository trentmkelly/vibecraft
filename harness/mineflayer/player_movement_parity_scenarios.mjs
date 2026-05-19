import { offlineUuid } from './runner.mjs'

export const PLAYER_MOVEMENT_PARITY_SCENARIOS = {
  playerState: [
    'health-sync',
    'food-sync',
    'saturation-visible-effects',
    'xp-sync',
    'game-mode-sync',
    'permissions-sync',
    'recipe-book-sync'
  ],
  gameModePersistence: [
    'survival-reconnect',
    'creative-reconnect',
    'adventure-reconnect',
    'spectator-reconnect',
    'force-gamemode-override',
    'saved-game-mode-restored'
  ],
  respawnAfterRelogin: [
    'death-screen-disconnect',
    'same-profile-reconnect',
    'death-state-restored',
    'respawn-state-recovered'
  ],
  deathRespawn: [
    'death-message',
    'respawn-packet-flow',
    'inventory-rules',
    'xp-rules',
    'spawn-position',
    'post-respawn-abilities'
  ],
  multiBotVisibility: [
    'two-bots-joined',
    'tab-list-entries',
    'spawn-despawn-packets',
    'relative-movement',
    'sneak-sprint-flags',
    'held-items',
    'disconnect-cleanup'
  ],
  entityTrackingDistance: [
    'tracking-threshold-enter-spawn',
    'metadata-timing',
    'velocity-timing',
    'equipment-timing',
    'tracking-threshold-exit-remove'
  ],
  invalidMovement: [
    'out-of-bounds',
    'too-fast',
    'illegal-stance',
    'no-clip',
    'flight-like',
    'correction-or-kick'
  ],
  movementTraceValidation: [
    'scripted-client-trace',
    'collision-trace',
    'vanilla-trace-comparison'
  ],
  movement: [
    'walking',
    'jumping',
    'sneaking',
    'sprinting',
    'falling',
    'invalid-correction',
    'chunk-boundary-crossing'
  ],
  teleportPositionConfirm: [
    'server-teleport',
    'relative-movement-flags',
    'yaw-pitch-correction',
    'cross-chunk-teleport',
    'dimension-change',
    'stale-teleport-confirmation'
  ]
}

export function createPlayerMovementParityPlan(kind, options = {}) {
  const steps = PLAYER_MOVEMENT_PARITY_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown player movement parity scenario ${kind}`)
  const username = options.username ?? defaultUsername(kind)
  return {
    name: `mineflayer-player-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gamemode ?? 'survival',
      difficulty: options.difficulty ?? 'normal'
    }
  }
}

export async function runPlayerMovementParityScenario(kind, options = {}) {
  const plan = createPlayerMovementParityPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? playerMovementProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing player movement evidence for ${step}`)
    recordPlayerMovement(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizePlayerMovement(evidence, plan)
  }
}

export function summarizePlayerMovement(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'player_movement_parity')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordPlayerMovement(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'player_movement_parity', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    playerState: 'PlayerStateBot',
    gameModePersistence: 'GameModePersistBot',
    respawnAfterRelogin: 'DeathRelogBot',
    deathRespawn: 'DeathRespawnBot',
    multiBotVisibility: 'VisibleOne',
    entityTrackingDistance: 'TrackingBot',
    invalidMovement: 'InvalidMoveBot',
    movementTraceValidation: 'MoveTraceBot',
    movement: 'MoveBot',
    teleportPositionConfirm: 'TeleportBot'
  }[kind] ?? 'PlayerBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function playerMovementProbe() {
  throw new Error('playerMovementProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'playerState'
  runPlayerMovementParityScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
