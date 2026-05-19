// Offline-mode death and respawn parity tests:
//   - kill bot, verify death message packet, respawn packet flow
//     (ClientboundRespawnPacket), inventory/XP rules on death
//     (keepInventory gamerule), respawn spawn position,
//     post-respawn ability flags
//   - kill bot, disconnect on death screen, reconnect; verify
//     vanilla-compatible death/respawn state recovery (no extra death,
//     correct spawn point)

export const DEATH_RESPAWN_SCENARIOS = {
  // Verify packet flow: death message → ClientboundRespawnPacket →
  // ability flags reset; keepInventory=false drops inventory/XP;
  // keepInventory=true retains them; post-respawn health is 20.
  deathRespawnFlow: [
    'death-message-received',
    'respawn-packet-received',
    'post-respawn-health-full',
    'keep-inventory-off-drops-items',
    'keep-inventory-on-retains-items',
    'keep-inventory-off-drops-xp',
    'keep-inventory-on-retains-xp',
    'post-respawn-ability-flags-match-game-mode'
  ],
  // Kill bot, disconnect before respawn button, reconnect; verify no
  // extra death recorded, spawn point used is correct.
  respawnAfterRelogin: [
    'killed-while-in-play',
    'disconnect-on-death-screen',
    'reconnect-reaches-play',
    'no-extra-death-on-reconnect',
    'respawn-position-matches-saved-spawn'
  ]
}

export function createDeathRespawnPlan(kind, options = {}) {
  const steps = DEATH_RESPAWN_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown death/respawn scenario ${kind}`)
  return {
    name: `mineflayer-death-respawn-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? defaultUsername(kind),
    steps,
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'normal',
      'spawn-protection': '0'
    }
  }
}

export function summarizeDeathRespawn(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(e => e.name === 'death_respawn')
    .map(e => e.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export async function runDeathRespawnScenario(kind, options = {}) {
  const plan = createDeathRespawnPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? deathRespawnProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing death/respawn evidence for ${step}`)
    recordDeathRespawn(evidence, step, result.details?.[step] ?? {})
  }
  return { plan, evidence, summary: summarizeDeathRespawn(evidence, plan) }
}

export function recordDeathRespawn(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'death_respawn', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    deathRespawnFlow: 'DeathRespawnBot',
    respawnAfterRelogin: 'RespawnReloginBot'
  }[kind] ?? 'DeathBot'
}

function kebab(str) {
  return str.replace(/([A-Z])/g, m => `-${m.toLowerCase()}`)
}

async function deathRespawnProbe(plan) {
  throw new Error(`deathRespawnProbe requires a scenario-specific server fixture for ${plan.kind}`)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'deathRespawnFlow'
  runDeathRespawnScenario(kind).then(result => {
    console.log(JSON.stringify(result.summary, null, 2))
  })
}
