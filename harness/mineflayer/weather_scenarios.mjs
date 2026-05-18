// Weather parity tests: rain/thunder transitions via /weather command,
// lightning ClientboundLevelEventPacket observed by bot,
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
      'lightning-level-event-packet',
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
    'lightning-level-event-packet': 'weather.lightning.level_event',
    'weather-command-feedback': 'weather.command.feedback',
    'reconnect-weather-state-preserved': 'weather.reconnect.preserved'
  }
  return map[step] ?? step
}

export function recordWeatherEvent(session, action, details = {}) {
  session.timeline.push({ name: 'weather', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Skylight reduction: CLEAR=0 (15 effective), RAIN/THUNDER=5 (10 effective)
export function effectiveSkyLight(raining, thundering) {
  return (raining || thundering) ? 10 : 15
}

// Vanilla lightning event ID used in ClientboundLevelEventPacket
export const LIGHTNING_LEVEL_EVENT_ID = 2005

export function planWeatherTransition(from, to, command) {
  const actionKey = `weather.transition.${from}_${to}`
  return { action: actionKey, from, to, command }
}

export function planLightningPacket(pos, expectedEventId) {
  return {
    action: 'weather.lightning.level_event',
    pos,
    expectedEventId: expectedEventId ?? LIGHTNING_LEVEL_EVENT_ID
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
