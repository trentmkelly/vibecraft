import test from 'node:test'
import assert from 'node:assert/strict'
import {
  checkChat,
  checkCommandExecution,
  checkConfigurationCompletion,
  checkDisconnectReason,
  checkKeepAlive,
  checkOfflineLogin,
  formatPacketFlowSmoke,
  runPacketFlowSmoke
} from './packet_flow_smoke.mjs'

test('runPacketFlowSmoke passes complete offline login packet flows', () => {
  const result = runPacketFlowSmoke(completeSession(), {
    chatMessage: 'hello',
    command: '/list'
  })
  assert.equal(result.ok, true)
  assert.deepEqual(result.checks.map(check => check.name), [
    'offlineLogin',
    'configurationCompletion',
    'keepAlive',
    'chat',
    'commandExecution',
    'disconnectReason'
  ])
})

test('offline login smoke check rejects interrupted login phases', () => {
  const session = completeSession()
  session.timeline = [
    { name: 'login', summary: [] },
    { name: 'kicked', summary: ['{"text":"Nope"}'] }
  ]
  assert.equal(checkOfflineLogin(session).ok, false)
})

test('configuration and keepalive checks report missing packets', () => {
  const session = completeSession()
  session.packetTrace = session.packetTrace.filter(packet => packet.name !== 'finish_configuration')
  assert.equal(checkConfigurationCompletion(session).ok, false)
  session.packetTrace = []
  assert.equal(checkKeepAlive(session).ok, false)
})

test('chat and command checks accept either packet traces or action events', () => {
  const session = completeSession()
  session.packetTrace = []
  assert.equal(checkChat(session, 'hello').ok, true)
  assert.equal(checkCommandExecution(session, '/list').ok, true)

  session.timeline = []
  session.packetTrace = [
    { state: 'play', name: 'player_chat', keys: ['message'], message: 'hello' },
    { state: 'play', name: 'chat_command', keys: ['command'], command: 'list' }
  ]
  assert.equal(checkChat(session, 'hello').ok, true)
  assert.equal(checkCommandExecution(session, '/list').ok, true)
})

test('disconnect reason check normalizes kicked events and disconnect packets', () => {
  const session = completeSession()
  session.timeline.push({ name: 'kicked', summary: ['{"text":"Server closed"}'] })
  assert.equal(checkDisconnectReason(session, 'Server closed').ok, true)
  assert.equal(checkDisconnectReason(session, 'Other reason').ok, false)
})

test('formatPacketFlowSmoke produces stable pass/fail lines', () => {
  const result = runPacketFlowSmoke(completeSession(), { command: '/list' })
  assert.match(formatPacketFlowSmoke(result), /PASS offlineLogin/)
  const failed = runPacketFlowSmoke({ timeline: [], packetTrace: [] })
  assert.match(formatPacketFlowSmoke(failed), /FAIL offlineLogin/)
})

function completeSession() {
  return {
    timeline: [
      { name: 'login', summary: [] },
      { name: 'packet', summary: ['success'] },
      { name: 'spawn', summary: [] },
      { name: 'message', summary: ['Bot says hello'] },
      { name: 'action', summary: ['issueCommand', { command: '/list' }] }
    ],
    packetTrace: [
      { direction: 'inbound', state: 'login', name: 'success', keys: ['uuid', 'username'] },
      { direction: 'inbound', state: 'configuration', name: 'registry_data', keys: ['registryCodec'] },
      { direction: 'outbound', state: 'configuration', name: 'finish_configuration', keys: [] },
      { direction: 'inbound', state: 'play', name: 'login', keys: ['entityId'] },
      { direction: 'inbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] },
      { direction: 'outbound', state: 'play', name: 'keep_alive', keys: ['keepAliveId'] },
      { direction: 'outbound', state: 'play', name: 'chat_command', keys: ['command'], command: 'list' }
    ]
  }
}
