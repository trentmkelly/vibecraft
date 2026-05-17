import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import {
  PROTOCOL_STATES,
  createPacketRecorder,
  normalizePacket,
  packetFlowSummary,
  replayPackets
} from './packet_recorder.mjs'

test('PROTOCOL_STATES covers status, login, configuration, and play flows', () => {
  assert.deepEqual(PROTOCOL_STATES, ['status', 'login', 'configuration', 'play'])
})

test('createPacketRecorder records normalized packet data with state filters', () => {
  const recorder = createPacketRecorder({ states: ['login', 'configuration'] })
  recorder.record({
    direction: 'inbound',
    state: 'login',
    name: 'success',
    at: 1,
    data: { username: 'Bot', uuid: 'u' }
  })
  recorder.record({ state: 'play', name: 'position', data: { x: 1 } })
  recorder.record({
    state: 'configuration',
    name: 'registry_data',
    data: { z: 1, a: Buffer.from([0xab]) }
  })
  assert.deepEqual(recorder.snapshot().map(packet => packet.name), ['success', 'registry_data'])
  assert.deepEqual(recorder.snapshot()[1].data, { a: { hex: 'ab' }, z: 1 })
})

test('attachClient records packet and optional raw events, then detaches cleanly', () => {
  const client = new EventEmitter()
  const recorder = createPacketRecorder()
  const detach = recorder.attachClient(client, { state: 'status', raw: true })
  client.emit('packet', { description: 'ok' }, { name: 'server_info', state: 'status' })
  client.emit('raw', Buffer.from([1, 2, 3]), { name: 'server_info', state: 'status' })
  detach()
  client.emit('packet', { ignored: true }, { name: 'ignored', state: 'status' })
  assert.deepEqual(packetFlowSummary(recorder.packets), [
    { direction: 'inbound', state: 'status', name: 'server_info', keys: ['description'] },
    { direction: 'inbound:raw', state: 'status', name: 'server_info', keys: ['length'] }
  ])
})

test('snapshot filters by state, packet name, and direction without mutating source packets', () => {
  const recorder = createPacketRecorder()
  recorder.record({ direction: 'outbound', state: 'login', name: 'hello', data: { username: 'Bot' } })
  recorder.record({ direction: 'inbound', state: 'play', name: 'keep_alive', data: { keepAliveId: 1 } })
  const snapshot = recorder.snapshot({ states: ['login'], directions: ['outbound'], names: ['hello'] })
  snapshot[0].data.username = 'Changed'
  assert.equal(snapshot.length, 1)
  assert.equal(recorder.packets[0].data.username, 'Bot')
})

test('replayPackets writes outbound packets and can emit inbound packets', async () => {
  const written = []
  const target = {
    write(name, data) {
      written.push([name, data])
    }
  }
  const packets = [
    normalizePacket({ direction: 'outbound', state: 'login', name: 'hello', data: { username: 'Bot' } }),
    normalizePacket({ direction: 'outbound', state: 'play', name: 'chat', data: { message: '/list' } })
  ]
  assert.equal(await replayPackets(packets, target, { states: ['login'] }), 1)
  assert.deepEqual(written, [['hello', { username: 'Bot' }]])

  const emitted = []
  const emitter = new EventEmitter()
  emitter.on('packet', (data, meta) => emitted.push([meta.name, meta.state, data]))
  assert.equal(await replayPackets(packets, emitter, { mode: 'emit', names: ['chat'] }), 1)
  assert.deepEqual(emitted, [['chat', 'play', { message: '/list' }]])
})

test('clear removes recorded packets for isolated scenario reuse', () => {
  const recorder = createPacketRecorder()
  recorder.record({ state: 'play', name: 'keep_alive', data: {} })
  recorder.clear()
  assert.deepEqual(recorder.packets, [])
})
