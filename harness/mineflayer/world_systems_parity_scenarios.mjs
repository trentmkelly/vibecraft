export const WORLD_SYSTEMS_PARITY_SCENARIOS = {
  gamerules: ['keep-inventory', 'do-immediate-respawn', 'send-command-feedback', 'do-daylight-cycle', 'mob-griefing'],
  statsAdvancements: ['movement-stat', 'mining-stat', 'crafting-stat', 'death-stat', 'recipe-unlock', 'client-updates', 'saved-json-after-reconnect'],
  scoreboardObjectiveLifecycle: ['create-objective', 'update-score', 'display-objective', 'hide-objective', 'persist-after-reconnect', 'remove-objective'],
  scoreboardTeams: ['sidebar-display', 'list-display', 'below-name-display', 'team-color-prefix-suffix', 'nametag-visibility', 'collision-rule', 'reconnect-persistence'],
  lootTableSmoke: ['custom-datapack-table', 'block-break-path', 'chest-open-path', 'loot-command-path', 'fishing-path', 'entity-death-path', 'advancement-reward-path', 'vanilla-diff'],
  lootContext: ['luck', 'tool', 'killer-player', 'origin', 'damage-source', 'explosion-radius', 'entity-properties', 'scoreboard-values', 'storage-nbt', 'random-sequence-id'],
  rewardSurface: ['chest-loot-refill-prevention', 'suspicious-block-brushing', 'piglin-bartering', 'cat-gifts', 'villager-gifts', 'fishing-catches', 'mob-equipment-drops', 'advancement-rewards', 'reconnect-persistence'],
  villagerTrading: ['merchant-window-open', 'offer-list-comparison', 'buy-items', 'exhaust-demand', 'restock-after-work-time', 'zombify-cure-discounts', 'reconnect-prices-xp'],
  xpRewards: ['mining-orbs', 'smelting-orbs', 'breeding-orbs', 'trading-orbs', 'command-orbs', 'mob-kill-orbs', 'advancement-orbs', 'level-bar-updates', 'orb-merge-timing', 'death-drops', 'reconnect-persistence'],
  trialRewards: ['trial-chamber-fixture', 'normal-spawner', 'ominous-spawner', 'vault-open', 'reconnect-mid-encounter', 'reward-drops', 'cooldowns', 'denied-open-feedback'],
  timeSleep: ['day-night-sync', 'bed-enter-leave', 'sleep-skipping', 'spawnpoint-setting', 'insomnia-counters', 'reconnect-visible-time'],
  weather: ['rain-transition', 'thunder-transition', 'lightning-observation', 'weather-command-feedback', 'client-state-after-reconnect'],
  worldBorder: ['initialize', 'lerp', 'warning-distance', 'warning-time', 'damage-buffer', 'damage-amount', 'movement-clamping', 'command-driven-updates']
}

export function createWorldSystemsParityPlan(kind, options = {}) {
  const steps = WORLD_SYSTEMS_PARITY_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown world systems scenario ${kind}`)
  return {
    name: `mineflayer-world-systems-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'WorldSystemsBot',
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runWorldSystemsParityScenario(kind, options = {}) {
  const plan = createWorldSystemsParityPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? worldSystemsProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing world systems evidence for ${step}`)
    recordWorldSystems(evidence, step, result.details?.[step] ?? {})
  }
  return { plan, evidence, summary: summarizeWorldSystems(evidence, plan) }
}

export function summarizeWorldSystems(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'world_systems_parity')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function recordWorldSystems(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'world_systems_parity', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function worldSystemsProbe() {
  throw new Error('worldSystemsProbe requires a scenario-specific server fixture')
}
