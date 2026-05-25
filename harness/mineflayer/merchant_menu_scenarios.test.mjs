import test from 'node:test'
import assert from 'node:assert/strict'
import {
  MERCHANT_MENU_SCENARIOS,
  createMerchantMenuScenarioPlan,
  recordMerchantMenuEvent,
  runMerchantMenuScenario,
  summarizeMerchantMenuEvidence
} from './merchant_menu_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

const REQUIRED_MERCHANT_SCENARIOS = [
  'openSendsThirtyNineSlots',
  'selectOfferAutoFillsPayment',
  'shiftClickTradeResult',
  'exhaustOffer',
  'rejectedTradeInputs',
  'staleOfferSelection',
  'xpAndPriceUpdateAfterTrade',
  'restockTiming',
  'closeReopenPreservesOfferState',
  'disconnectMidTradeReturnsPayments',
  'closeWhileCarryingReturnsItems'
]

test('merchant menu scenario plan covers the checklist behaviors', () => {
  for (const kind of REQUIRED_MERCHANT_SCENARIOS) {
    assert.ok(MERCHANT_MENU_SCENARIOS[kind], `missing scenario ${kind}`)
    assert.ok(MERCHANT_MENU_SCENARIOS[kind].length > 0, `scenario ${kind} has no steps`)
  }
})

for (const kind of Object.keys(MERCHANT_MENU_SCENARIOS)) {
  test(`createMerchantMenuScenarioPlan covers ${kind}`, () => {
    const plan = createMerchantMenuScenarioPlan(kind)
    assert.equal(plan.name, `mineflayer-merchant-menu-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.kind, kind)
    assert.equal(plan.mode, 'offline')
    assert.equal(plan.auth, 'offline')
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.deepEqual(plan.steps, MERCHANT_MENU_SCENARIOS[kind])
    assert.equal(plan.serverProperties['online-mode'], 'false')
  })

  test(`runMerchantMenuScenario validates all ${kind} steps`, async () => {
    const result = await runMerchantMenuScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { observed: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
    for (const step of result.plan.steps) {
      assert.equal(result.summary.steps[step], true)
    }
  })
}

test('createMerchantMenuScenarioPlan throws on unknown scenario kinds', () => {
  assert.throws(() => createMerchantMenuScenarioPlan('does-not-exist'), /Unknown merchant menu scenario/)
})

test('summarizeMerchantMenuEvidence fails on missing evidence', () => {
  const plan = createMerchantMenuScenarioPlan('openSendsThirtyNineSlots')
  const evidence = { timeline: [] }
  recordMerchantMenuEvent(evidence, plan.steps[0])
  // Only the first step is recorded — the remaining ones are missing.
  const summary = summarizeMerchantMenuEvidence(evidence, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps[plan.steps[0]], true)
  assert.equal(summary.steps[plan.steps[1]], false)
})

test('runMerchantMenuScenario throws when probe omits a step', async () => {
  await assert.rejects(
    () => runMerchantMenuScenario('shiftClickTradeResult', {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.slice(0, -1).map(step => [step, true]))
      })
    }),
    /Missing merchant menu evidence/
  )
})

test('default merchant probe rejects until villager entity is implemented', async () => {
  await assert.rejects(
    () => runMerchantMenuScenario('openSendsThirtyNineSlots'),
    /requires a live villager entity/
  )
})
