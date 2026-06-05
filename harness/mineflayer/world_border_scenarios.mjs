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

// Border damage (LivingEntity.tick / WorldBorder, 26.1.2): when the entity is
// outside the safe zone (dist + safeZone < 0, here distanceOutsideBuffer = -dist),
// the applied damage is max(1, floor(distanceOutside * damagePerBlock)) and 0 when
// inside. Vanilla floors the product and enforces a minimum of 1 HP per tick; it is
// NOT a raw 0.2*distance float. damagePerBlock defaults to 0.2.
export function computeBorderDamage(distanceOutsideBuffer, damagePerBlock = 0.2) {
  if (distanceOutsideBuffer <= 0 || damagePerBlock <= 0) return 0
  return Math.max(1, Math.floor(distanceOutsideBuffer * damagePerBlock))
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
    expectedDamage: computeBorderDamage(testDistanceOutside, damagePerBlock)
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('world_border_scenarios defined — run via the test harness')
}
