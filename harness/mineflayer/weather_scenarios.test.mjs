import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createWeatherScenariosPlan,
  summarizeWeatherScenarios,
  effectiveSkyLight,
  LIGHTNING_LEVEL_EVENT_ID,
  VANILLA_WEATHER_FEEDBACK,
  planWeatherTransition,
  planLightningPacket,
  planWeatherCommandFeedback
} from './weather_scenarios.mjs'

test('createWeatherScenariosPlan covers all 6 weather steps', () => {
  const plan = createWeatherScenariosPlan()
  assert.equal(plan.name, 'mineflayer-weather-scenarios')
  assert.equal(plan.steps.length, 6)
  assert.ok(plan.steps.includes('clear-to-rain-transition'))
  assert.ok(plan.steps.includes('lightning-level-event-packet'))
  assert.ok(plan.steps.includes('reconnect-weather-state-preserved'))
})

test('summarizeWeatherScenarios returns ok when all steps evidenced', () => {
  const plan = createWeatherScenariosPlan()
  const actionMap = {
    'clear-to-rain-transition': 'weather.transition.clear_rain',
    'rain-to-thunder-transition': 'weather.transition.rain_thunder',
    'thunder-to-clear-transition': 'weather.transition.thunder_clear',
    'lightning-level-event-packet': 'weather.lightning.level_event',
    'weather-command-feedback': 'weather.command.feedback',
    'reconnect-weather-state-preserved': 'weather.reconnect.preserved'
  }
  const timeline = plan.steps.map(step => ({
    name: 'weather', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeWeatherScenarios({ timeline }, plan).ok, true)
})

test('summarizeWeatherScenarios fails with empty timeline', () => {
  const plan = createWeatherScenariosPlan()
  assert.equal(summarizeWeatherScenarios({ timeline: [] }, plan).ok, false)
})

test('effectiveSkyLight returns 15 for clear and 10 for rain or thunder', () => {
  assert.equal(effectiveSkyLight(false, false), 15)  // clear
  assert.equal(effectiveSkyLight(true, false), 10)   // rain
  assert.equal(effectiveSkyLight(true, true), 10)    // thunderstorm
  assert.equal(effectiveSkyLight(false, true), 10)   // thunder (raining flag may be separate)
})

test('LIGHTNING_LEVEL_EVENT_ID is 2005', () => {
  assert.equal(LIGHTNING_LEVEL_EVENT_ID, 2005)
})

test('VANILLA_WEATHER_FEEDBACK covers clear, rain, thunder', () => {
  assert.equal(VANILLA_WEATHER_FEEDBACK.clear, 'commands.weather.set.clear')
  assert.equal(VANILLA_WEATHER_FEEDBACK.rain, 'commands.weather.set.rain')
  assert.equal(VANILLA_WEATHER_FEEDBACK.thunder, 'commands.weather.set.thunder')
})

test('planWeatherTransition records from/to states', () => {
  const p = planWeatherTransition('clear', 'rain', '/weather rain')
  assert.equal(p.action, 'weather.transition.clear_rain')
  assert.equal(p.from, 'clear')
  assert.equal(p.to, 'rain')
  assert.equal(p.command, '/weather rain')
})

test('planLightningPacket uses default event ID when omitted', () => {
  const p = planLightningPacket({ x: 0, y: 64, z: 0 })
  assert.equal(p.expectedEventId, LIGHTNING_LEVEL_EVENT_ID)
})

test('planWeatherCommandFeedback records type and key', () => {
  const p = planWeatherCommandFeedback('thunder', 'commands.weather.set.thunder')
  assert.equal(p.weatherType, 'thunder')
  assert.equal(p.expectedFeedbackKey, 'commands.weather.set.thunder')
})
