import test from 'node:test'
import assert from 'node:assert/strict'
import {
  chatCommandScenario,
  chatCommandScenarioManifest,
  chatScenarioManifest,
  observeChatScenario,
  offlineChatScenarios,
  summarizeChatCommandEvidence
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

test('chatCommandScenarioManifest covers chat fallbacks, feedback, suggestions, and tab completion', () => {
  const manifest = chatCommandScenarioManifest({ sender: 'Steve', target: 'Alex' })

  assert.deepEqual(manifest.steps, [
    'signed-chat-fallback',
    'unsigned-chat-fallback',
    'system-messages',
    'command-feedback',
    'suggestions',
    'tab-completion'
  ])
  assert.ok(manifest.commands.includes('/tell Alex secret'))
  assert.equal(manifest.target, 'Alex')
})

test('summarizeChatCommandEvidence requires chat fallbacks, feedback, suggestions, and tab completion', () => {
  const scenario = chatCommandScenario({ sender: 'Steve', target: 'Alex' })
  const evidence = {
    timeline: [
      { name: 'chat_command', summary: ['signed-chat-fallback', { accepted: true }] },
      { name: 'chat_command', summary: ['unsigned-chat-fallback', { accepted: true }] },
      { name: 'chat_command', summary: ['suggestions', { includesRoot: true }] },
      { name: 'chat_command', summary: ['tab-completion', { includesTarget: 'Alex' }] },
      { name: 'message', summary: ['commands.message.display'] },
      { name: 'message', summary: ['commands.me.success'] }
    ]
  }

  assert.equal(summarizeChatCommandEvidence(evidence, scenario).ok, true)
  evidence.timeline[3].summary[1].includesTarget = 'Steve'
  assert.equal(summarizeChatCommandEvidence(evidence, scenario).ok, false)
})
