import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import {
  assertCarriedItemCorrection,
  assertHeldItemSync,
  clickWindowSlot,
  createInventoryScenarioPlan,
  dropHeldItem,
  openAndCloseWindow,
  selectHotbarSlot,
  waitForItemPickup
} from './inventory_scenarios.mjs'

test('selectHotbarSlot uses Mineflayer quick bar selection and records held item', async () => {
  const session = fakeSession()
  await selectHotbarSlot(session, 2)
  assert.equal(session.bot.quickBarSlot, 2)
  assert.deepEqual(session.timeline[0].summary, [
    'hotbar.select',
    {
      slot: 2,
      heldItem: { name: 'stick', count: 3, displayName: 'Stick', type: 280 }
    }
  ])
  await assert.rejects(() => selectHotbarSlot(session, 9), /Hotbar slot/)
})

test('waitForItemPickup waits for playerCollect and verifies inventory delta', async () => {
  const session = fakeSession()
  const pickup = waitForItemPickup(session, { name: 'emerald', count: 2 }, { timeoutMs: 100 })
  session.bot.inventory.slots[10] = item('emerald', 2)
  session.bot.emit('playerCollect', { id: 1, username: 'VibeCraftBot' }, { id: 2, name: 'item' })
  const result = await pickup
  assert.equal(result.assertion.ok, true)
  assert.equal(result.assertion.count, 2)
  assert.equal(result.collected.name, 'item')
})

test('dropHeldItem covers full-stack and partial Mineflayer drop helpers', async () => {
  const session = fakeSession()
  await dropHeldItem(session)
  assert.deepEqual(session.bot.calls[0], ['tossStack', item('stick', 3)])

  await dropHeldItem(session, { fullStack: false, count: 1 })
  assert.deepEqual(session.bot.calls[1], ['toss', 280, null, 1])
  assert.deepEqual(session.timeline.map(event => event.summary[0]), ['item.drop', 'item.drop'])
})

test('openAndCloseWindow opens target block and closes the returned window', async () => {
  const session = fakeSession()
  const result = await openAndCloseWindow(session, { x: 4, y: 65, z: 6 })
  assert.deepEqual(session.bot.calls[0], ['openBlock', { x: 4, y: 65, z: 6 }])
  assert.equal(session.windowClosed, true)
  assert.equal(result.opened.type, 'minecraft:chest')
})

test('clickWindowSlot records slot before and after the Mineflayer click', async () => {
  const session = fakeSession()
  const window = fakeWindow()
  const result = await clickWindowSlot(session, window, 5, 1, 0)
  assert.deepEqual(session.bot.calls[0], ['clickWindow', 5, 1, 0])
  assert.deepEqual(result.before, { name: 'apple', count: 4, displayName: 'Apple', type: 260 })
  assert.deepEqual(result.after, { name: 'apple', count: 1, displayName: 'Apple', type: 260 })
})

test('assertHeldItemSync checks held item against selected hotbar backing slot', () => {
  const session = fakeSession()
  const ok = assertHeldItemSync(session, { name: 'stick', count: 3 })
  assert.equal(ok.ok, true)
  session.bot.inventory.slots[38] = item('stone', 1)
  const mismatch = assertHeldItemSync(session, { name: 'stick', count: 3 })
  assert.equal(mismatch.ok, false)
  assert.equal(mismatch.inventoryItem.name, 'stone')
})

test('assertCarriedItemCorrection compares cursor and authoritative server slot', () => {
  const session = fakeSession()
  session.bot.inventory.cursor = item('diamond', 1)
  session.bot.inventory.slots[12] = item('air', 0)
  const result = assertCarriedItemCorrection(session, {
    slot: 12,
    carried: { name: 'diamond', count: 1 },
    serverSlot: { name: 'air', count: 0 }
  })
  assert.equal(result.ok, true)
})

test('createInventoryScenarioPlan keeps inventory coverage as separate executable steps', () => {
  const plan = createInventoryScenarioPlan({
    hotbarSlot: 4,
    pickup: { name: 'emerald', count: 2 },
    windowSlot: 13
  })
  assert.deepEqual(plan.steps.map(step => step.action), [
    'hotbar.select',
    'item.pickup',
    'item.drop',
    'window.open_close',
    'window.click_slot',
    'held_item.sync',
    'carried_item.correction'
  ])
  assert.equal(plan.steps[0].slot, 4)
  assert.equal(plan.steps[1].expected.name, 'emerald')
  assert.equal(plan.steps[4].slot, 13)
})

function fakeSession() {
  const bot = new EventEmitter()
  bot.quickBarSlot = 2
  bot.heldItem = item('stick', 3)
  bot.inventory = { slots: [], cursor: null, selectedItem: null }
  bot.inventory.slots[38] = item('stick', 3)
  bot.calls = []
  bot.setQuickBarSlot = slot => {
    bot.quickBarSlot = slot
  }
  bot.tossStack = async stack => {
    bot.calls.push(['tossStack', stack])
    return 'dropped stack'
  }
  bot.toss = async (type, metadata, count) => {
    bot.calls.push(['toss', type, metadata, count])
    return 'dropped item'
  }
  bot.blockAt = position => ({ position, name: 'chest' })
  bot.openBlock = async block => {
    bot.calls.push(['openBlock', block.position])
    return fakeWindow(() => { session.windowClosed = true })
  }
  bot.clickWindow = async (slot, mouseButton, mode) => {
    bot.calls.push(['clickWindow', slot, mouseButton, mode])
    return 'clicked'
  }
  const session = { bot, timeline: [], windowClosed: false }
  return session
}

function fakeWindow(onClose = () => {}) {
  const window = {
    id: 1,
    type: 'minecraft:chest',
    title: 'Chest',
    slots: [],
    close: onClose
  }
  let clicked = false
  Object.defineProperty(window.slots, '5', {
    configurable: true,
    enumerable: true,
    get() {
      if (!clicked) {
        clicked = true
        return item('apple', 4)
      }
      return item('apple', 1)
    }
  })
  return window
}

function item(name, count) {
  const data = {
    air: { type: null, displayName: null },
    apple: { type: 260, displayName: 'Apple' },
    diamond: { type: 264, displayName: 'Diamond' },
    emerald: { type: 388, displayName: 'Emerald' },
    stick: { type: 280, displayName: 'Stick' },
    stone: { type: 1, displayName: 'Stone' }
  }[name] ?? { type: null, displayName: name }
  return { name, count, displayName: data.displayName, type: data.type }
}
