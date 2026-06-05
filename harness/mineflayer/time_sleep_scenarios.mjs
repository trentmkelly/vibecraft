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

// Insomnia threshold (PhantomSpawner, 26.1.2): a phantom can spawn only when
//   random.nextInt(clamp(timeSinceRest, 1, MAX)) >= 72000
// nextInt(n) maxes at n-1, so timeSinceRest must be STRICTLY greater than 72000
// (>= 72001) for the comparison to ever hold; at exactly 72000 it never spawns.
export const INSOMNIA_PHANTOM_THRESHOLD_TICKS = 72000

// Sleep eligibility (BedRule.CAN_SLEEP_WHEN_DARK -> Rule.WHEN_DARK ->
// Level.isDarkOutside()). 26.1.2 no longer uses a hardcoded daytime window; it is
// derived from skyDarken:
//   Level.skyDarken = (int)(15.0F - SKY_LIGHT_LEVEL)
//   Level.isBrightOutside() = skyDarken < 4 ; isDarkOutside() = !bright
// SKY_LIGHT_LEVEL = 15 (default) * MULTIPLY-track value, where the track
// (Timelines.java:81-83, LINEAR easing) is keyframed
//   (133,1.0)(11867,1.0)(13670,0.26666668)(22330,0.26666668).
// Computed in float (Math.fround) to match the vanilla f32 arithmetic exactly.
const DAY_LENGTH_TICKS = 24000
const SKY_LIGHT_LEVEL_DEFAULT = 15.0
const SKY_LIGHT_MULTIPLY_KEYFRAMES = [
  [133, 1.0],
  [11867, 1.0],
  [13670, 0.26666668],
  [22330, 0.26666668]
]

// Cyclic piecewise-linear sampler matching environment_attributes::sample_piecewise_linear.
function samplePiecewiseLinear(dayCycleTicks, keyframes) {
  const t = ((dayCycleTicks % DAY_LENGTH_TICKS) + DAY_LENGTH_TICKS) % DAY_LENGTH_TICKS
  const n = keyframes.length
  let prevTick, prevVal, nextTick, nextVal
  if (t < keyframes[0][0]) {
    prevTick = keyframes[n - 1][0] - DAY_LENGTH_TICKS
    prevVal = keyframes[n - 1][1]
    nextTick = keyframes[0][0]
    nextVal = keyframes[0][1]
  } else if (t >= keyframes[n - 1][0]) {
    prevTick = keyframes[n - 1][0]
    prevVal = keyframes[n - 1][1]
    nextTick = keyframes[0][0] + DAY_LENGTH_TICKS
    nextVal = keyframes[0][1]
  } else {
    let idx = 0
    while (idx + 1 < n && keyframes[idx + 1][0] <= t) idx++
    prevTick = keyframes[idx][0]
    prevVal = keyframes[idx][1]
    nextTick = keyframes[idx + 1][0]
    nextVal = keyframes[idx + 1][1]
  }
  const span = nextTick - prevTick
  if (span <= 0) return Math.fround(nextVal)
  const frac = Math.fround((t - prevTick) / span)
  return Math.fround(prevVal + Math.fround(frac * Math.fround(nextVal - prevVal)))
}

export function computeSkyLightLevel(dayTime) {
  return Math.fround(SKY_LIGHT_LEVEL_DEFAULT * samplePiecewiseLinear(dayTime, SKY_LIGHT_MULTIPLY_KEYFRAMES))
}

// Level.skyDarken = (int)(15 - SKY_LIGHT_LEVEL); (int) truncates toward zero.
export function computeSkyDarken(dayTime) {
  return Math.trunc(Math.fround(SKY_LIGHT_LEVEL_DEFAULT - computeSkyLightLevel(dayTime)))
}

// Level.isDarkOutside() == skyDarken >= 4 (yielding the sleep window 12523..23477).
export function isNightTimeForSleep(dayTime) {
  return computeSkyDarken(dayTime) >= 4
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
    // random.nextInt(value) >= 72000 requires value > 72000 (nextInt(72000) maxes
    // at 71999), so phantoms become eligible strictly above the threshold.
    expectPhantomSpawn: ticksWithoutSleep > INSOMNIA_PHANTOM_THRESHOLD_TICKS
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('time_sleep_scenarios defined — run via the test harness')
}
