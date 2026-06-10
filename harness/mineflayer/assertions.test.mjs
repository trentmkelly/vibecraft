import test from 'node:test'
import assert from 'node:assert/strict'
import {
  assertEventOrder,
  assertKickedMessage,
  assertLoginPhases,
  assertNoUnexpectedKick,
  assertOfflineUuid,
  assertPacketOrder,
  assertParityDiff,
  formatAssertionFailure,
  formatParityDiff,
  normalizeKickedMessage
} from './assertions.mjs'

test('assertOfflineUuid verifies fixture and actual UUIDs against vanilla derivation', () => {
  assert.equal(assertOfflineUuid({
    username: 'Steve',
    expectedUuid: '5627dd98-e6be-3c21-b8a8-e92344183641',
    actualUuid: '5627DD98-E6BE-3C21-B8A8-E92344183641'
  }).ok, true)
  const bad = assertOfflineUuid({ username: 'Steve', actualUuid: '00000000-0000-0000-0000-000000000000' })
  assert.equal(bad.ok, false)
  assert.equal(bad.code, 'uuid.actual_mismatch')
})

test('assertLoginPhases and assertEventOrder accept packet noise between phases', () => {
  const events = [
    { name: 'packet', summary: ['login'] },
    { name: 'login', summary: [] },
    { name: 'packet', summary: ['game_profile'] },
    { name: 'spawn', summary: [] }
  ]
  assert.equal(assertLoginPhases(events).ok, true)
  assert.equal(assertEventOrder(events, ['login', 'spawn'], { exact: true, allowPacketsBetween: true }).ok, true)
  assert.equal(assertEventOrder(events, ['spawn', 'login']).ok, false)
  assert.equal(assertLoginPhases([{ name: 'login' }, { name: 'kicked', summary: ['Nope'] }]).code, 'login.interrupted')
})

test('assertPacketOrder checks ordered packet subsequences', () => {
  const events = [
    { name: 'packet', summary: ['login'] },
    { name: 'message', summary: ['hello'] },
    { name: 'packet', summary: ['configuration'] },
    { name: 'packet', summary: ['position'] }
  ]
  assert.equal(assertPacketOrder(events, ['login', 'position']).ok, true)
  assert.equal(assertPacketOrder(events, ['position', 'login']).code, 'packet.missing')
})

test('kicked message normalization handles JSON components, formatting, and whitespace', () => {
  assert.equal(
    normalizeKickedMessage('{"translate":"multiplayer.disconnect.kicked"}'),
    'multiplayer.disconnect.kicked'
  )
  assert.equal(
    normalizeKickedMessage({ text: 'Disconnected: ', extra: [{ text: 'bad login' }] }),
    'Disconnected: bad login'
  )
  assert.equal(normalizeKickedMessage(' §cNope\\nagain '), 'Nope again')
  assert.equal(assertKickedMessage([
    { name: 'kicked', summary: ['{"text":"Server closed"}'] }
  ], 'Server closed').ok, true)
  assert.equal(assertNoUnexpectedKick([{ name: 'spawn' }]).ok, true)
  assert.equal(assertNoUnexpectedKick([{ name: 'kicked', summary: ['{"text":"bye"}'] }]).code, 'kick.unexpected')
})

test('assertParityDiff allows expected surfaces and reports unexpected ones', () => {
  assert.equal(assertParityDiff([{ path: 'logs' }], { allowedPaths: ['logs'] }).ok, true)
  const result = assertParityDiff([{ path: 'events', official: [], rebuilt: [] }])
  assert.equal(result.ok, false)
  assert.match(formatAssertionFailure(result), /parity.diff/)
  assert.match(formatParityDiff([{ path: 'events', official: ['spawn'], rebuilt: ['kicked'] }]), /VibeCraft/)
  assert.equal(formatParityDiff([]), 'official and VibeCraft artifacts match')
})
