import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile, rm } from 'node:fs/promises'
import path from 'node:path'
import {
  createTempWorld,
  diffArtifacts,
  normalizeArtifacts,
  offlineUuid,
  runParityScenario,
  startOfficialServer,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

test('offlineUuid matches vanilla generated UUIDs', () => {
  assert.equal(offlineUuid('Steve'), '5627dd98-e6be-3c21-b8a8-e92344183641')
  assert.equal(offlineUuid('Alex'), '36532b5e-c442-3dbb-a24c-c7e55d0f979a')
})

test('writeOfflineServerFiles creates eula and offline-mode properties', async () => {
  const root = await createTempWorld('rustcraft-mf-files-')
  try {
    await writeOfflineServerFiles(root, {
      port: 31234,
      seed: 123,
      levelName: 'login-world',
      properties: { 'max-players': '3' }
    })
    assert.equal(await readFile(path.join(root, 'eula.txt'), 'utf8'), 'eula=true\n')
    const properties = await readFile(path.join(root, 'server.properties'), 'utf8')
    assert.match(properties, /^online-mode=false$/m)
    assert.match(properties, /^enforce-secure-profile=false$/m)
    assert.match(properties, /^server-port=31234$/m)
    assert.match(properties, /^level-seed=123$/m)
    assert.match(properties, /^level-name=login-world$/m)
    assert.match(properties, /^max-players=3$/m)
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

test('startRustCraft builds vanilla-compatible CLI arguments and captures logs', async () => {
  const root = await createTempWorld('rustcraft-mf-spawn-')
  const script = path.join(root, 'fake-server.mjs')
  await import('node:fs/promises').then(fs =>
    fs.writeFile(script, "console.log('ready'); setTimeout(() => {}, 5000)\n")
  )
  const server = startRustCraft({
    binary: process.execPath,
    root,
    levelName: 'world',
    port: 30001,
    env: {}
  })
  try {
    assert.deepEqual(server.args, [
      '--nogui',
      '--universe', root,
      '--world', 'world',
      '--port', '30001'
    ])
  } finally {
    await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

test('startOfficialServer builds java -jar command and captures logs', async () => {
  const root = await createTempWorld('rustcraft-mf-official-test-')
  const server = startOfficialServer({
    java: process.execPath,
    jar: '/tmp/server.jar',
    root,
    javaArgs: ['--version']
  })
  try {
    assert.deepEqual(server.args, ['--version', '-jar', '/tmp/server.jar', '--nogui'])
  } finally {
    await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

test('waitForPort fails fast with actionable endpoint details', async () => {
  await assert.rejects(
    () => waitForPort(9, '127.0.0.1', 50),
    /Timed out waiting for 127\.0\.0\.1:9/
  )
})

test('normalizeArtifacts redacts roots, ports, and timestamps before diffing', () => {
  const official = normalizeArtifacts({
    root: '/tmp/a',
    events: [{ name: 'spawn', at: 1, summary: ['ok'] }],
    logs: [{ stream: 'stdout', text: '2026-05-17T12:00:00Z started /tmp/a on 25565\n' }],
    serverProperties: 'server-port=25565\nlevel-seed=123\n',
    eula: 'eula=true\n'
  }, { root: '/tmp/a', port: 25565 })
  const rebuilt = normalizeArtifacts({
    root: '/tmp/b',
    events: [{ name: 'spawn', at: 2, summary: ['ok'] }],
    logs: [{ stream: 'stdout', text: '2026-05-17T12:00:01Z started /tmp/b on 25566\n' }],
    serverProperties: 'level-seed=123\nserver-port=25566\n',
    eula: 'eula=true\n'
  }, { root: '/tmp/b', port: 25566 })
  assert.deepEqual(diffArtifacts(official, rebuilt), [])
})

test('diffArtifacts reports observable parity differences by surface', () => {
  const diffs = diffArtifacts(
    { events: [{ name: 'spawn', summary: [] }], logs: [], serverProperties: '', eula: 'eula=true\n' },
    { events: [{ name: 'kicked', summary: ['no'] }], logs: [], serverProperties: '', eula: 'eula=true\n' }
  )
  assert.equal(diffs.length, 1)
  assert.equal(diffs[0].path, 'events')
})

test('runParityScenario runs identical script through official and RustCraft adapters', async () => {
  const root = await createTempWorld('rustcraft-mf-parity-')
  const officialRoot = path.join(root, 'official')
  const rebuiltRoot = path.join(root, 'rebuilt')
  const fake = path.join(root, 'fake-server.mjs')
  await import('node:fs/promises').then(fs =>
    fs.writeFile(fake, "console.log('ready'); setTimeout(() => {}, 5000)\n")
  )
  try {
    const result = await runParityScenario({
      jar: fake,
      java: process.execPath,
      binary: process.execPath,
      officialRoot,
      rebuiltRoot,
      port: 33333,
      rebuiltPort: 33334,
      timeoutMs: 25,
      keepArtifacts: false
    })
    assert.equal(result.equivalent, false)
    assert.ok(result.diff.some(entry => entry.path === 'logs' || entry.path === 'serverProperties'))
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})
