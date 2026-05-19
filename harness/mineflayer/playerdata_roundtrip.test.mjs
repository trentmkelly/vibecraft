import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PLAYERDATA_ROUNDTRIP_FIELDS,
  assertRoundTrip,
  createPlayerdataRoundTripPlan,
  recordPlayerdataRoundTrip,
  runPlayerdataRoundTrip,
  summarizePlayerdataRoundTrip
} from './playerdata_roundtrip.mjs'
import { offlineUuid } from './runner.mjs'

test('createPlayerdataRoundTripPlan covers every requested playerdata field', () => {
  const plan = createPlayerdataRoundTripPlan()
  assert.equal(plan.name, 'mineflayer-playerdata-round-trip')
  assert.equal(plan.uuid, offlineUuid(plan.username))
  assert.deepEqual(plan.fields, PLAYERDATA_ROUNDTRIP_FIELDS)
  for (const field of PLAYERDATA_ROUNDTRIP_FIELDS) {
    assert.ok(field in plan.beforeDisconnect, `${field} should be staged before disconnect`)
  }
})

test('summarizePlayerdataRoundTrip requires changed, saved, and restored evidence', () => {
  const plan = createPlayerdataRoundTripPlan()
  const evidence = { timeline: [] }
  recordPlayerdataRoundTrip(evidence, 'state-changed-before-disconnect')
  recordPlayerdataRoundTrip(evidence, 'playerdata-saved-after-disconnect')
  assert.equal(summarizePlayerdataRoundTrip(evidence, plan).ok, false)
  recordPlayerdataRoundTrip(evidence, 'state-restored-after-reconnect')
  assert.equal(summarizePlayerdataRoundTrip(evidence, plan).ok, true)
})

test('runPlayerdataRoundTrip validates full save and reconnect restore', async () => {
  const plan = createPlayerdataRoundTripPlan()
  const fields = plan.beforeDisconnect
  const result = await runPlayerdataRoundTrip({
    changeStateProbe: async () => ({ fields }),
    saveProbe: async activePlan => ({ saved: true, uuid: activePlan.uuid, fields }),
    reconnectProbe: async () => ({ loadedBeforeSpawn: true, fields })
  })
  assert.equal(result.summary.ok, true)
})

test('assertRoundTrip fails on selected slot mismatch', () => {
  const plan = createPlayerdataRoundTripPlan()
  assert.throws(() => assertRoundTrip(plan.beforeDisconnect, {
    ...plan.beforeDisconnect,
    selectedSlot: plan.beforeDisconnect.selectedSlot + 1
  }), /selectedSlot mismatch/)
})

test('runPlayerdataRoundTrip fails when recipe book is missing from saved data', async () => {
  const plan = createPlayerdataRoundTripPlan()
  const fields = { ...plan.beforeDisconnect }
  delete fields.recipeBook
  await assert.rejects(() => runPlayerdataRoundTrip({
    changeStateProbe: async () => ({ fields: plan.beforeDisconnect }),
    saveProbe: async activePlan => ({ saved: true, uuid: activePlan.uuid, fields }),
    reconnectProbe: async () => ({ loadedBeforeSpawn: true, fields: plan.beforeDisconnect })
  }), /Missing recipeBook/)
})
