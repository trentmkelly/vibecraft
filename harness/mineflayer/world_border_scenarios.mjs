// World border parity tests: init packet (size, center), lerp interpolation,
// warning distance/time from packets, damage buffer/amount, movement clamping,
// command-driven updates.

export function createWorldBorderScenariosPlan(options = {}) {
  return {
    name: 'mineflayer-world-border-scenarios',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BorderBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    steps: [
      'init-border-size-center-packet',
      'lerp-new-size-with-time',
      'warning-distance-packet',
      'warning-time-packet',
      'damage-outside-buffer',
      'movement-clamped-at-border',
      'command-update-sends-packets'
    ]
  }
}

export function summarizeWorldBorderScenarios(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'border')
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
    'init-border-size-center-packet': 'border.init.packet',
    'lerp-new-size-with-time': 'border.lerp.size_time',
    'warning-distance-packet': 'border.warning.distance',
    'warning-time-packet': 'border.warning.time',
    'damage-outside-buffer': 'border.damage.outside_buffer',
    'movement-clamped-at-border': 'border.movement.clamped',
    'command-update-sends-packets': 'border.command.update'
  }
  return map[step] ?? step
}

export function recordBorderEvent(session, action, details = {}) {
  session.timeline.push({ name: 'border', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Border damage formula: 0.2 * max(0, distance_outside_buffer)
export function computeBorderDamage(distanceOutsideBuffer) {
  return 0.2 * Math.max(0, distanceOutsideBuffer)
}

export function planBorderInit(expectedSize, expectedCenterX, expectedCenterZ) {
  return {
    action: 'border.init.packet',
    expectedSize,
    expectedCenterX,
    expectedCenterZ
  }
}

export function planBorderLerp(oldSize, newSize, lerpMillis) {
  return {
    action: 'border.lerp.size_time',
    oldSize,
    newSize,
    lerpMillis,
    expectLerping: lerpMillis > 0 && oldSize !== newSize
  }
}

export function planBorderWarning(warningDistanceBlocks, warningTimeSecs) {
  return {
    distanceAction: { action: 'border.warning.distance', warningDistanceBlocks },
    timeAction: { action: 'border.warning.time', warningTimeSecs }
  }
}

export function planBorderDamage(bufferBlocks, damagePerBlock, testDistanceOutside) {
  return {
    action: 'border.damage.outside_buffer',
    bufferBlocks,
    damagePerBlock,
    testDistanceOutside,
    expectedDamage: computeBorderDamage(testDistanceOutside)
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('world_border_scenarios defined — run via the test harness')
}
