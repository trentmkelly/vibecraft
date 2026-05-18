// Time/sleep parity tests: day-time sync (ClientboundSetTimePacket each tick),
// bed enter/leave sequence, sleep skipping (all players → skip to morning),
// spawnpoint set on sleep, insomnia counter (72000 ticks), reconnect-visible time.

export function createTimeSleepScenariosPlan(options = {}) {
  return {
    name: 'mineflayer-time-sleep-scenarios',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'SleepBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    },
    steps: [
      'daytime-sync-set-time-packet',
      'bed-enter-sequence',
      'bed-leave-sequence',
      'sleep-skip-to-morning',
      'spawnpoint-set-on-sleep',
      'insomnia-counter-72000-ticks',
      'reconnect-time-visible'
    ]
  }
}

export function summarizeTimeSleepScenarios(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'time')
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
    'daytime-sync-set-time-packet': 'time.sync.set_time_packet',
    'bed-enter-sequence': 'time.bed.enter',
    'bed-leave-sequence': 'time.bed.leave',
    'sleep-skip-to-morning': 'time.sleep.skip_morning',
    'spawnpoint-set-on-sleep': 'time.sleep.spawnpoint_set',
    'insomnia-counter-72000-ticks': 'time.insomnia.counter',
    'reconnect-time-visible': 'time.reconnect.visible'
  }
  return map[step] ?? step
}

export function recordTimeEvent(session, action, details = {}) {
  session.timeline.push({ name: 'time', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Sleep skip must advance to next 24000 boundary, not just 0
export function expectedMorningTimeAfterSleep(currentTime) {
  const fullDay = 24000
  const cycleTime = currentTime % fullDay
  if (cycleTime === 0) return currentTime + fullDay
  return currentTime + (fullDay - cycleTime)
}

// Insomnia threshold: 72000 ticks without sleep triggers phantom spawning
export const INSOMNIA_PHANTOM_THRESHOLD_TICKS = 72000

// Vanilla daytime range for sleep eligibility: 12541–23458
export function isNightTimeForSleep(dayTime) {
  return dayTime >= 12541 && dayTime <= 23458
}

export function planBedEnter(username, bedPos) {
  return { action: 'time.bed.enter', username, bedPos }
}

export function planSleepSkip(timeBeforeSleep, expectedMorningTime) {
  return { action: 'time.sleep.skip_morning', timeBeforeSleep, expectedMorningTime }
}

export function planInsomniaCounter(ticksWithoutSleep) {
  return {
    action: 'time.insomnia.counter',
    ticksWithoutSleep,
    expectPhantomSpawn: ticksWithoutSleep >= INSOMNIA_PHANTOM_THRESHOLD_TICKS
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('time_sleep_scenarios defined — run via the test harness')
}
