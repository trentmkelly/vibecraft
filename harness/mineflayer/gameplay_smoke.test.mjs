import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createGameplaySmokePlan,
  summarizeGameplaySmoke
} from './gameplay_smoke.mjs'

test('createGameplaySmokePlan covers Mineflayer movement, block, item, inventory, respawn, and reconnect workflows', () => {
  const plan = createGameplaySmokePlan({ username: 'SmokeBot' })

  assert.equal(plan.name, 'mineflayer-gameplay-smoke')
  assert.equal(plan.client, 'mineflayer')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.equal(plan.username, 'SmokeBot')
  assert.equal(plan.serverProperties['online-mode'], 'false')
  assert.deepEqual(plan.steps, [
    'spawn',
    'movement',
    'block-dig',
    'block-place',
    'item-pickup',
    'item-drop',
    'inventory-click',
    'respawn',
    'reconnect-persistence'
  ])
})

test('summarizeGameplaySmoke accepts complete gameplay smoke evidence', () => {
  const plan = createGameplaySmokePlan()
  const session = {
    timeline: [
      { name: 'spawn' },
      { name: 'death' },
      { name: 'action', summary: ['movement', {}] },
      { name: 'block', summary: ['block.dig', {}] },
      { name: 'block', summary: ['block.place', {}] },
      { name: 'inventory', summary: ['item.pickup', {}] },
      { name: 'inventory', summary: ['item.drop', {}] },
      { name: 'inventory', summary: ['window.click_slot', {}] },
      { name: 'action', summary: ['forceReconnect.connected', {}] },
      { name: 'action', summary: ['reconnectPersistence', {}] }
    ]
  }

  const summary = summarizeGameplaySmoke(session, plan)
  assert.equal(summary.ok, true)
  assert.deepEqual(Object.values(summary.steps), [
    true,
    true,
    true,
    true,
    true,
    true,
    true,
    true,
    true
  ])
})

test('summarizeGameplaySmoke fails missing required workflow evidence', () => {
  const summary = summarizeGameplaySmoke({
    timeline: [
      { name: 'spawn' },
      { name: 'action', summary: ['movement', {}] }
    ]
  }, createGameplaySmokePlan())

  assert.equal(summary.ok, false)
  assert.equal(summary.steps['block-dig'], false)
  assert.equal(summary.steps['reconnect-persistence'], false)
})
