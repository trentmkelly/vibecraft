// Mineflayer randomizable-container parity plan: open generated containers
// before and after reconnect, verifying one-time loot realization plus lock,
// custom name, and comparator-visible inventory state.

export const RANDOMIZABLE_CONTAINER_TYPES = [
  'chest',
  'barrel',
  'dispenser',
  'dropper',
  'shulker_box'
]

export function createRandomizableContainerPlan(options = {}) {
  const containers = options.containers ?? RANDOMIZABLE_CONTAINER_TYPES
  return {
    name: 'mineflayer-randomizable-containers',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'RandomizableContainerBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    containers,
    steps: containers.flatMap(container => [
      `${container}:open-before-reconnect`,
      `${container}:open-after-reconnect`
    ]),
    assertions: {
      lootRealization: 'exactly-once',
      customName: 'visible-on-open',
      lock: 'enforced-before-open',
      comparatorContents: 'matches-realized-inventory'
    }
  }
}

export function summarizeRandomizableContainerRun(session, plan) {
  const actions = new Set(
    (session.timeline ?? [])
      .filter(event => event.name === 'randomizable_container')
      .map(event => event.summary?.[0])
  )
  const steps = Object.fromEntries(
    plan.steps.map(step => [step, actions.has(stepToAction(step))])
  )
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordRandomizableContainerEvent(session, action, details = {}) {
  session.timeline.push({
    name: 'randomizable_container',
    at: Date.now(),
    summary: [action, details]
  })
  return { action, ...details }
}

export function planRandomizableContainerCase(container, options = {}) {
  if (!RANDOMIZABLE_CONTAINER_TYPES.includes(container)) {
    throw new Error(`Unsupported randomizable container: ${container}`)
  }
  return {
    container,
    lootTable: options.lootTable ?? 'minecraft:chests/simple_dungeon',
    customName: options.customName ?? `VibeCraft ${container}`,
    lockKey: options.lockKey ?? 'vibecraft_key',
    beforeReconnectAction: `randomizable_container.${container}.before_reconnect`,
    afterReconnectAction: `randomizable_container.${container}.after_reconnect`,
    expected: {
      realizationCount: 1,
      lockedOpenDenied: true,
      unlockedOpenAllowed: true,
      customNameVisible: true,
      comparatorMatchesContents: true
    }
  }
}

function stepToAction(step) {
  const [container, phase] = step.split(':')
  const suffix = phase === 'open-before-reconnect' ? 'before_reconnect' : 'after_reconnect'
  return `randomizable_container.${container}.${suffix}`
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('randomizable container scenarios defined — run via the test harness')
}
