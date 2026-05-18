// Status effect parity tests: apply effect, tick duration countdown, stack amplifier,
// expire (remove packet), clear via milk bucket, save/restore on reconnect,
// client-visible particles/icons/amplifiers/durations.

export function createStatusEffectScenariosPlan(options = {}) {
  return {
    name: 'mineflayer-status-effect-scenarios',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'EffectBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    steps: [
      'apply-effect-packet-received',
      'tick-duration-countdown',
      'stack-amplifier-higher-replaces',
      'expire-remove-packet-sent',
      'clear-via-milk-bucket',
      'save-restore-on-reconnect',
      'client-particles-icons-amplifiers-durations'
    ]
  }
}

export function summarizeStatusEffectScenarios(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'effect')
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
    'apply-effect-packet-received': 'effect.apply.packet',
    'tick-duration-countdown': 'effect.tick.countdown',
    'stack-amplifier-higher-replaces': 'effect.stack.amplifier',
    'expire-remove-packet-sent': 'effect.expire.remove_packet',
    'clear-via-milk-bucket': 'effect.clear.milk',
    'save-restore-on-reconnect': 'effect.persistence.reconnect',
    'client-particles-icons-amplifiers-durations': 'effect.client.display'
  }
  return map[step] ?? step
}

export function recordEffectEvent(session, action, details = {}) {
  session.timeline.push({ name: 'effect', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function planApplyEffect(effectId, amplifier, durationTicks) {
  return { action: 'effect.apply.packet', effectId, amplifier, durationTicks }
}

export function planEffectStack(effectId, lowAmplifier, highAmplifier) {
  return {
    action: 'effect.stack.amplifier',
    effectId,
    lowAmplifier,
    highAmplifier,
    expectHigherWins: true
  }
}

export function planEffectExpire(effectId, remainingTicks) {
  return { action: 'effect.expire.remove_packet', effectId, remainingTicks }
}

export function planEffectPersistence(effectsToSave) {
  return {
    action: 'effect.persistence.reconnect',
    effects: effectsToSave,
    expectRestored: true
  }
}

// Ambient flag: beacon-given effects use fewer particles (alpha 38 vs 255)
export function ambientParticleAlpha(ambient) {
  return ambient ? 38 : 255
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('status_effect_scenarios defined — run via the test harness')
}
