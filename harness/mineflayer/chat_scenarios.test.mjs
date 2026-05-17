import test from 'node:test'
import assert from 'node:assert/strict'
import {
  chatScenarioManifest,
  observeChatScenario,
  offlineChatScenarios
} from './chat_scenarios.mjs'

test('offlineChatScenarios cover public, private, formatted, death, advancement, and malformed disconnect cases', () => {
  assert.deepEqual(offlineChatScenarios().map(scenario => scenario.name), [
    'publicChat',
    'privateMessage',
    'formattedChat',
    'deathAnnouncement',
    'advancementAnnouncement',
    'malformedDisconnect'
  ])
})

test('observeChatScenario matches public, private, and formatted chat messages', () => {
  const session = {
    timeline: [
      { name: 'message', summary: ['RustCraftBot: hello'] },
      { name: 'message', summary: ['RustCraftBot whispers to you: secret'] },
      { name: 'message', summary: ['<RustCraftBot> colored text'] }
    ]
  }
  for (const scenario of offlineChatScenarios().slice(0, 3)) {
    assert.equal(observeChatScenario(session, scenario).ok, true)
  }
})

test('observeChatScenario treats death and advancement announcements as optional where unavailable', () => {
  const session = { timeline: [] }
  assert.equal(observeChatScenario(session, offlineChatScenarios()[3]).ok, true)
  assert.equal(observeChatScenario(session, offlineChatScenarios()[4]).ok, true)
})

test('observeChatScenario normalizes malformed-message disconnect reasons', () => {
  const scenario = offlineChatScenarios().find(entry => entry.name === 'malformedDisconnect')
  const session = {
    timeline: [{
      name: 'kicked',
      summary: ['{"text":"multiplayer.disconnect.invalid_packet"}']
    }]
  }
  assert.deepEqual(observeChatScenario(session, scenario), {
    ok: true,
    reason: 'multiplayer.disconnect.invalid_packet'
  })
})

test('observeChatScenario reports missing required chat messages', () => {
  assert.equal(observeChatScenario({ timeline: [] }, offlineChatScenarios()[0]).ok, false)
})

test('chatScenarioManifest serializes offline chat metadata', () => {
  const manifest = chatScenarioManifest({ sender: 'Steve', target: 'Alex' })
  assert.equal(manifest.mode, 'offline')
  assert.equal(manifest.auth, 'offline')
  assert.equal(manifest.scenarios.length, 6)
  assert.ok(manifest.scenarios.some(scenario => scenario.name === 'privateMessage'))
})
