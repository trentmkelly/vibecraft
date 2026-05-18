import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createTimeSleepScenariosPlan,
  summarizeTimeSleepScenarios,
  expectedMorningTimeAfterSleep,
  isNightTimeForSleep,
  INSOMNIA_PHANTOM_THRESHOLD_TICKS,
  planBedEnter,
  planSleepSkip,
  planInsomniaCounter
} from './time_sleep_scenarios.mjs'

test('createTimeSleepScenariosPlan covers all 7 time/sleep steps', () => {
  const plan = createTimeSleepScenariosPlan()
  assert.equal(plan.name, 'mineflayer-time-sleep-scenarios')
  assert.equal(plan.steps.length, 7)
  assert.ok(plan.steps.includes('daytime-sync-set-time-packet'))
  assert.ok(plan.steps.includes('sleep-skip-to-morning'))
  assert.ok(plan.steps.includes('insomnia-counter-72000-ticks'))
})

test('summarizeTimeSleepScenarios returns ok when all steps evidenced', () => {
  const plan = createTimeSleepScenariosPlan()
  const actionMap = {
    'daytime-sync-set-time-packet': 'time.sync.set_time_packet',
    'bed-enter-sequence': 'time.bed.enter',
    'bed-leave-sequence': 'time.bed.leave',
    'sleep-skip-to-morning': 'time.sleep.skip_morning',
    'spawnpoint-set-on-sleep': 'time.sleep.spawnpoint_set',
    'insomnia-counter-72000-ticks': 'time.insomnia.counter',
    'reconnect-time-visible': 'time.reconnect.visible'
  }
  const timeline = plan.steps.map(step => ({
    name: 'time', summary: [actionMap[step], {}]
  }))
  assert.equal(summarizeTimeSleepScenarios({ timeline }, plan).ok, true)
})

test('summarizeTimeSleepScenarios fails with empty timeline', () => {
  const plan = createTimeSleepScenariosPlan()
  assert.equal(summarizeTimeSleepScenarios({ timeline: [] }, plan).ok, false)
})

test('expectedMorningTimeAfterSleep skips to next 24000 boundary, not 0', () => {
  // Night of day 0: t=13000 → next boundary is 24000
  assert.equal(expectedMorningTimeAfterSleep(13000), 24000)
  // Night of day 1: t=37000 → next boundary is 48000
  assert.equal(expectedMorningTimeAfterSleep(37000), 48000)
  // Exactly at day boundary: t=0 → next boundary is 24000
  assert.equal(expectedMorningTimeAfterSleep(0), 24000)
  // Exactly at boundary: t=24000 → next boundary is 48000
  assert.equal(expectedMorningTimeAfterSleep(24000), 48000)
})

test('isNightTimeForSleep identifies valid sleep window 12541-23458', () => {
  assert.equal(isNightTimeForSleep(12541), true)   // start of night
  assert.equal(isNightTimeForSleep(18000), true)   // midnight
  assert.equal(isNightTimeForSleep(23458), true)   // end of night
  assert.equal(isNightTimeForSleep(12540), false)  // just before night
  assert.equal(isNightTimeForSleep(23459), false)  // just after night
  assert.equal(isNightTimeForSleep(6000), false)   // noon
})

test('INSOMNIA_PHANTOM_THRESHOLD_TICKS is 72000', () => {
  assert.equal(INSOMNIA_PHANTOM_THRESHOLD_TICKS, 72000)
})

test('planBedEnter records username and position', () => {
  const p = planBedEnter('SleepBot', { x: 10, y: 64, z: 20 })
  assert.equal(p.action, 'time.bed.enter')
  assert.equal(p.username, 'SleepBot')
  assert.deepEqual(p.bedPos, { x: 10, y: 64, z: 20 })
})

test('planSleepSkip records before/after times', () => {
  const p = planSleepSkip(13000, 24000)
  assert.equal(p.action, 'time.sleep.skip_morning')
  assert.equal(p.timeBeforeSleep, 13000)
  assert.equal(p.expectedMorningTime, 24000)
})

test('planInsomniaCounter flags phantom spawn at threshold', () => {
  const below = planInsomniaCounter(71999)
  assert.equal(below.expectPhantomSpawn, false)
  const at = planInsomniaCounter(72000)
  assert.equal(at.expectPhantomSpawn, true)
  const above = planInsomniaCounter(90000)
  assert.equal(above.expectPhantomSpawn, true)
})
