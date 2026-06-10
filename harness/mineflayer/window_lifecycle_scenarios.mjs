import { offlineUuid } from './runner.mjs'

/// Mineflayer scenarios that exercise the open → click → close → reopen
/// lifecycle for every menu type the server can build.
///
/// Each menu kind has a fixed slot count (defined by the vanilla
/// `AbstractContainerMenu` subclasses) that the bot must observe on the
/// initial `ContainerSetContent` packet. The plan steps are intentionally
/// granular so that a future test rig can answer "did the server send
/// ContainerSetContent with the correct slot count?" and "did the click
/// echo back through ContainerSetSlot?" independently.
export const WINDOW_LIFECYCLE_MENUS = {
  playerInventory: { slotCount: 46, openOnJoin: true },
  chest: { slotCount: 27 + 36, openOnJoin: false },
  furnace: { slotCount: 3 + 36, openOnJoin: false },
  craftingTable: { slotCount: 10 + 36, openOnJoin: false },
  anvil: { slotCount: 3 + 36, openOnJoin: false },
  merchant: { slotCount: 3 + 36, openOnJoin: false }
}

export const WINDOW_LIFECYCLE_STEPS = [
  'open-window',
  'observe-container-set-content',
  'observe-correct-slot-count',
  'click-slot',
  'observe-container-set-slot-echo',
  'close-window',
  'reopen-window',
  'observe-preserved-state',
  'disconnect-mid-window',
  'observe-items-saved-or-dropped'
]

export function createWindowLifecyclePlan(menu, options = {}) {
  const config = WINDOW_LIFECYCLE_MENUS[menu]
  if (!config) throw new Error(`Unknown window lifecycle menu ${menu}`)
  const username = options.username ?? defaultUsername(menu)
  return {
    name: `mineflayer-window-lifecycle-${kebab(menu)}`,
    menu,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    slotCount: config.slotCount,
    openOnJoin: config.openOnJoin,
    steps: config.openOnJoin
      // Player inventory is implicitly open from the join sequence, so we
      // skip the explicit `open-window` step for that menu.
      ? WINDOW_LIFECYCLE_STEPS.filter(step => step !== 'open-window')
      : WINDOW_LIFECYCLE_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gamemode ?? 'survival'
    }
  }
}

export async function runWindowLifecycleScenario(menu, options = {}) {
  const plan = createWindowLifecyclePlan(menu, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? windowLifecycleProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing window lifecycle evidence for ${menu}:${step}`)
    recordWindowLifecycleEvent(evidence, step, result.details?.[step] ?? {})
  }
  // Sanity check: the probe must report the menu's slot count so we know it
  // actually observed `ContainerSetContent`. This guards against probes that
  // only echo back the requested step names without inspecting traffic.
  if (result.slotCount !== plan.slotCount) {
    throw new Error(
      `Slot count mismatch for ${menu}: probe reported ${result.slotCount}, expected ${plan.slotCount}`
    )
  }
  return {
    plan,
    evidence,
    summary: summarizeWindowLifecycleEvidence(evidence, plan)
  }
}

export function summarizeWindowLifecycleEvidence(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'window_lifecycle')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordWindowLifecycleEvent(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'window_lifecycle', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(menu) {
  return {
    playerInventory: 'WinLifePlayerBot',
    chest: 'WinLifeChestBot',
    furnace: 'WinLifeFurnaceBot',
    craftingTable: 'WinLifeCraftBot',
    anvil: 'WinLifeAnvilBot',
    merchant: 'WinLifeMerchantBot'
  }[menu] ?? 'WinLifeBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function windowLifecycleProbe() {
  throw new Error(
    'windowLifecycleProbe requires a live VibeCraft server and an in-world block (or villager) to interact with; pass an `options.probe` override to drive the scenario manually'
  )
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const menu = process.argv[2] ?? 'chest'
  runWindowLifecycleScenario(menu).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
