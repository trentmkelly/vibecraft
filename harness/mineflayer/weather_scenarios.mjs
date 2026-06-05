// Weather parity tests: rain/thunder transitions via /weather command,
// lightning LightningBolt entity spawn observed by bot,
// weather command feedback, client state after reconnect.

export function createWeatherScenariosPlan(options = {}) {
  return {
    name: 'mineflayer-weather-scenarios',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'WeatherBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    steps: [
      'clear-to-rain-transition',
      'rain-to-thunder-transition',
      'thunder-to-clear-transition',
      'lightning-bolt-entity-spawned',
      'weather-command-feedback',
      'reconnect-weather-state-preserved'
    ]
  }
}

export function summarizeWeatherScenarios(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'weather')
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
    'clear-to-rain-transition': 'weather.transition.clear_rain',
    'rain-to-thunder-transition': 'weather.transition.rain_thunder',
    'thunder-to-clear-transition': 'weather.transition.thunder_clear',
    'lightning-bolt-entity-spawned': 'weather.lightning.entity_spawn',
    'weather-command-feedback': 'weather.command.feedback',
    'reconnect-weather-state-preserved': 'weather.reconnect.preserved'
  }
  return map[step] ?? step
}

export function recordWeatherEvent(session, action, details = {}) {
  session.timeline.push({ name: 'weather', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Effective sky light during weather = 15 - skyDarken, where skyDarken is the
// weather contribution at full intensity (environment_attributes::weather_sky_darken,
// WeatherAttributes ALPHA_BLEND toward SKY_LIGHT_LEVEL 4.0):
//   clear  -> skyDarken 0 -> light 15
//   rain   -> skyDarken 3 -> light 12
//   thunder-> skyDarken 5 -> light 10  (thunder implies rain)
// Rain and thunder are NOT both 10; rain only dims to 12.
export function effectiveSkyLight(raining, thundering) {
  if (thundering) return 10
  if (raining) return 12
  return 15
}

// Lightning is observed as a spawned LightningBolt ENTITY (EntityType "lightning_bolt",
// ServerLevel.tickThunder -> addFreshEntity), surfaced to the client via
// ClientboundAddEntityPacket — NOT a ClientboundLevelEventPacket. (Level event 2005 is
// bonemeal particles, unrelated.) The bot observes it as an entitySpawn of this type.
export const LIGHTNING_BOLT_ENTITY_ID = 'minecraft:lightning_bolt'

export function planWeatherTransition(from, to, command) {
  const actionKey = `weather.transition.${from}_${to}`
  return { action: actionKey, from, to, command }
}

export function planLightningSpawn(pos, expectedEntityType) {
  return {
    action: 'weather.lightning.entity_spawn',
    pos,
    expectedEntityType: expectedEntityType ?? LIGHTNING_BOLT_ENTITY_ID
  }
}

export function planWeatherCommandFeedback(weatherType, expectedFeedbackKey) {
  return {
    action: 'weather.command.feedback',
    weatherType,
    expectedFeedbackKey
  }
}

// Weather command feedback keys match vanilla localization
export const VANILLA_WEATHER_FEEDBACK = {
  clear: 'commands.weather.set.clear',
  rain: 'commands.weather.set.rain',
  thunder: 'commands.weather.set.thunder'
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('weather_scenarios defined — run via the test harness')
}
