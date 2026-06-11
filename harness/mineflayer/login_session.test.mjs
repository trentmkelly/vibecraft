import test from 'node:test'
import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import { createTempWorld, offlineUuid } from './runner.mjs'
import {
  captureObservedBotEvents,
  loginSessionPaths,
  runObservedOfflineLogin
} from './login_session.mjs'

test('loginSessionPaths names all temp artifacts used by offline login sessions', () => {
  assert.deepEqual(loginSessionPaths('/tmp/vibecraft-login', 'login-world'), {
    root: '/tmp/vibecraft-login',
    eula: '/tmp/vibecraft-login/eula.txt',
    serverProperties: '/tmp/vibecraft-login/server.properties',
    world: '/tmp/vibecraft-login/login-world'
  })
})

test('captureObservedBotEvents records event timeline and packet trace separately', () => {
  const bot = new EventEmitter()
  bot._client = new EventEmitter()
  bot._client.state = 'play'
  const timeline = []
  const packetTrace = []
  captureObservedBotEvents(bot, timeline, packetTrace)

  bot.emit('login')
  bot._client.emit('packet', { entityId: 1, gameMode: 1 }, { name: 'login', state: 'play' })
  bot._client.emit('error', new Error('Bad packet id 1'))
  bot.emit('kicked', '{"text":"unexpected play packet 1"}')
  bot.emit('spawn')

  assert.deepEqual(timeline.map(event => event.name), ['login', 'packet', 'packet_error', 'kicked', 'spawn'])
  assert.deepEqual(packetTrace, [
    {
      name: 'login',
      state: 'play',
      at: packetTrace[0].at,
      keys: ['entityId', 'gameMode']
    },
    {
      name: 'client_error',
      state: 'play',
      at: packetTrace[1].at,
      message: 'Bad packet id 1'
    }
  ])
  assert.deepEqual(timeline[3].summary, ['{"text":"unexpected play packet 1"}'])
})

test('runObservedOfflineLogin returns profile, UUID, events, packets, paths, logs, and cleanup handles', async () => {
  const root = await createTempWorld('vibecraft-mf-login-test-')
  let cleanupCalled = false
  let server
  const session = await runObservedOfflineLogin({
    root,
    port: 30125,
    username: 'Steve',
    levelName: 'login-world',
    keepAlive: true,
    keepArtifacts: true,
    startServer: () => {
      server = fakeServer()
      return server
    },
    waitForReady: async () => {
      server.logs.push({ stream: 'stdout', text: 'ready\n' })
    },
    connectBot: async options => {
      options.timeline.push({ name: 'login', at: 1, summary: [] })
      options.packetTrace.push({ name: 'success', state: 'login', at: 2, keys: ['uuid', 'username'] })
      options.timeline.push({ name: 'packet', at: 2, summary: ['success', 'login'] })
      options.timeline.push({ name: 'spawn', at: 3, summary: [] })
      return {
        bot: { end: () => { cleanupCalled = true } },
        timeline: options.timeline,
        packetTrace: options.packetTrace,
        profile: {
          username: options.username,
          expectedUuid: offlineUuid(options.username),
          actualUuid: offlineUuid(options.username)
        }
      }
    }
  })
  try {
    assert.equal(session.profile.username, 'Steve')
    assert.equal(session.uuid, '5627dd98-e6be-3c21-b8a8-e92344183641')
    assert.deepEqual(session.timeline.map(event => event.name), ['login', 'packet', 'spawn'])
    assert.equal(session.events, session.timeline)
    assert.deepEqual(session.packetTrace.map(packet => packet.name), ['success'])
    assert.equal(session.paths.world.endsWith('login-world'), true)
    assert.deepEqual(session.serverLogs, [{ stream: 'stdout', text: 'ready\n' }])
    assert.equal(typeof session.cleanup, 'function')
    assert.equal(typeof session.handles.stopServer, 'function')
    assert.equal(typeof session.handles.endBot, 'function')
    const artifacts = await session.collectArtifacts()
    assert.equal(artifacts.events.length, 3)
    assert.match(artifacts.serverProperties, /^online-mode=false$/m)
  } finally {
    await session.cleanup()
    await session.handles.removeArtifacts()
  }
  assert.equal(cleanupCalled, true)
})

test('runObservedOfflineLogin returns observed failure sessions with cleanup handles', async () => {
  const session = await runObservedOfflineLogin({
    port: 30126,
    username: 'Alex',
    autoCleanup: true,
    startServer: () => fakeServer(),
    waitForReady: async () => {
      throw new Error('not ready')
    }
  })
  assert.match(session.error.message, /not ready/)
  assert.equal(session.profile.expectedUuid, offlineUuid('Alex'))
  assert.equal(session.packetTrace.length, 0)
  assert.equal(typeof session.cleanup, 'function')
})

function fakeServer() {
  const child = new EventEmitter()
  child.stdin = { write: () => {} }
  child.exitCode = 0
  child.signalCode = null
  return {
    child,
    logs: [],
    args: []
  }
}
