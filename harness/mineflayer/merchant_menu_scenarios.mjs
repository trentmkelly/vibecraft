import { offlineUuid } from './runner.mjs'

/// Mineflayer scenarios that exercise the `MerchantMenu` protocol surface.
///
/// These are "plan" scenarios — the merchant entity is not yet wired into the
/// RustCraft server, so the tests describe the expected outcomes for each
/// step rather than driving a real bot through them. When a real villager
/// entity becomes available, the scenarios here become the integration
/// reference: the names and step ordering match the assertions the bot is
/// expected to produce against a live server.
export const MERCHANT_MENU_SCENARIOS = {
  // Happy-path: open a merchant window, observe the 39-slot ContainerSetContent.
  openSendsThirtyNineSlots: [
    'open-merchant-window',
    'observe-container-set-content-39-slots',
    'observe-merchant-offers-packet'
  ],

  // SelectTradePacket → tradeContainer auto-fills the payment slots from
  // the player's inventory (Java: MerchantMenu.tryMoveItems).
  selectOfferAutoFillsPayment: [
    'open-merchant-window',
    'send-select-trade-packet',
    'observe-payment-slot-a-autofill',
    'observe-payment-slot-b-autofill-when-cost-b-present',
    'observe-result-slot-shows-offer-output'
  ],

  // Shift-click the result slot → quickMoveStack moves the offer output
  // into the player inventory and increments `uses`.
  shiftClickTradeResult: [
    'open-merchant-window',
    'select-trade',
    'shift-click-result-slot',
    'observe-result-moved-to-player-inventory',
    'observe-uses-incremented',
    'observe-payment-deducted'
  ],

  // Trading until uses == maxUses leaves the result slot empty and
  // `MerchantContainer::can_trade()` returns false.
  exhaustOffer: [
    'open-merchant-window',
    'select-trade',
    'trade-until-out-of-stock',
    'observe-result-slot-empty',
    'observe-offer-flagged-out-of-stock'
  ],

  // Closing the merchant window while carrying items returns the cursor
  // and the payment slots to the player inventory (Java:
  // MerchantMenu.removed → Inventory.placeItemBackInInventory).
  closeWhileCarryingReturnsItems: [
    'open-merchant-window',
    'place-payment-items',
    'pick-up-payment-with-cursor',
    'close-window',
    'observe-cursor-returned-to-inventory',
    'observe-payment-slots-returned-to-inventory'
  ]
}

export function createMerchantMenuScenarioPlan(kind, options = {}) {
  const steps = MERCHANT_MENU_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown merchant menu scenario ${kind}`)
  const username = options.username ?? defaultUsername(kind)
  return {
    name: `mineflayer-merchant-menu-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gamemode ?? 'survival'
    }
  }
}

export async function runMerchantMenuScenario(kind, options = {}) {
  const plan = createMerchantMenuScenarioPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? merchantMenuProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing merchant menu evidence for ${step}`)
    recordMerchantMenuEvent(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizeMerchantMenuEvidence(evidence, plan)
  }
}

export function summarizeMerchantMenuEvidence(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'merchant_menu')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordMerchantMenuEvent(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'merchant_menu', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(kind) {
  return {
    openSendsThirtyNineSlots: 'MerchantOpenBot',
    selectOfferAutoFillsPayment: 'MerchantSelectBot',
    shiftClickTradeResult: 'MerchantShiftClickBot',
    exhaustOffer: 'MerchantExhaustBot',
    closeWhileCarryingReturnsItems: 'MerchantCloseBot'
  }[kind] ?? 'MerchantBot'
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function merchantMenuProbe() {
  throw new Error(
    'merchantMenuProbe requires a live villager entity; pass an `options.probe` override to drive the scenario manually'
  )
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'openSendsThirtyNineSlots'
  runMerchantMenuScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
