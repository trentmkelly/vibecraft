// Offline-mode spawnpoint-persistence tests: set bed and respawn-anchor spawn points,
// reconnect, die, respawn; verify saved spawn state and missing-spawn fallback match vanilla.

export function createSpawnpointPersistencePlan(options = {}) {
  return {
    name: 'mineflayer-spawnpoint-persistence',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'SpawnBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    compareAgainst: ['vanilla-26.1.2', 'rustcraft'],
    assertions: [
      'bed-spawn-saved-to-playerdata',
      'anchor-spawn-saved-to-playerdata',
      'spawnpoint-restored-before-first-visible-spawn-after-reconnect',
      'death-respawn-uses-valid-bed-spawn',
      'death-respawn-uses-charged-anchor-spawn',
      'missing-or-invalid-spawn-falls-back-to-world-spawn'
    ],
    steps: [
      'set-bed-spawnpoint',
      'set-respawn-anchor-spawnpoint',
      'reconnect-spawnpoint-restored',
      'die-respawn-at-bed',
      'die-respawn-at-anchor',
      'missing-spawn-fallback-world-spawn'
    ]
  }
}

export function summarizeSpawnpointPersistence(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'spawn')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, actions.includes(stepToAction(step))])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function stepToAction(step) {
  const map = {
    'set-bed-spawnpoint': 'spawn.bed.set',
    'set-respawn-anchor-spawnpoint': 'spawn.anchor.set',
    'reconnect-spawnpoint-restored': 'spawn.reconnect.restored',
    'die-respawn-at-bed': 'spawn.die.bed_respawn',
    'die-respawn-at-anchor': 'spawn.die.anchor_respawn',
    'missing-spawn-fallback-world-spawn': 'spawn.fallback.world_spawn'
  }
  return map[step] ?? step
}

export function recordSpawnEvent(session, action, details = {}) {
  session.timeline.push({ name: 'spawn', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function planBedSpawnSet(username, bedPos, dimension) {
  return {
    action: 'spawn.bed.set',
    username,
    bedPos,
    dimension,
    expectedSavedFields: {
      spawn_dimension: dimension,
      spawn_x: bedPos.x,
      spawn_y: bedPos.y,
      spawn_z: bedPos.z,
      spawn_forced: false
    }
  }
}

export function planAnchorSpawnSet(username, anchorPos, dimension, charges = 4) {
  return {
    action: 'spawn.anchor.set',
    username,
    anchorPos,
    dimension,
    charges,
    expectedSavedFields: {
      spawn_dimension: dimension,
      spawn_x: anchorPos.x,
      spawn_y: anchorPos.y,
      spawn_z: anchorPos.z,
      spawn_forced: false
    }
  }
}

export function planDeathRespawn(spawnType, expectedPos) {
  return {
    action: spawnType === 'bed' ? 'spawn.die.bed_respawn' : 'spawn.die.anchor_respawn',
    spawnType,
    expectedPos,
    assertions: [
      'death-screen-observed',
      'client-respawn-packet-observed',
      'post-respawn-position-matches-spawnpoint',
      'abilities-and-game-mode-restored-after-respawn'
    ]
  }
}

// Vanilla fallback: if bed/anchor missing or obstructed, respawn at world spawn
export function planMissingSpawnFallback(reason) {
  return {
    action: 'spawn.fallback.world_spawn',
    reason,
    assertions: [
      'invalid-spawn-state-cleared-or-ignored',
      'world-spawn-position-used',
      'missing-spawn-message-or-packet-matches-vanilla'
    ]
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('spawnpoint_persistence defined — run via the test harness')
}
