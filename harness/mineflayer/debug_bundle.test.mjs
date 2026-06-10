import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { createTempWorld } from './runner.mjs'
import { formatLogs, writeLoginDebugBundle } from './debug_bundle.mjs'

test('formatLogs preserves stream labels and raw text', () => {
  assert.equal(formatLogs([
    { stream: 'stdout', text: 'ready\n' },
    { stream: 'stderr', text: 'warn\n' }
  ]), '[stdout] ready\n[stderr] warn\n')
})

test('writeLoginDebugBundle writes profile, timeline, packets, logs, artifacts, and parity diff', async () => {
  const root = await createTempWorld('vibecraft-mf-debug-root-')
  const outputRoot = await createTempWorld('vibecraft-mf-debug-out-')
  const world = path.join(root, 'world')
  await mkdir(world, { recursive: true })
  await writeFile(path.join(world, 'level.dat'), 'fake')
  await writeFile(path.join(root, 'server.properties'), 'server-port=25565\n')
  await writeFile(path.join(root, 'eula.txt'), 'eula=true\n')

  const session = {
    root,
    paths: {
      world,
      serverProperties: path.join(root, 'server.properties'),
      eula: path.join(root, 'eula.txt')
    },
    endpoint: { port: 25565 },
    profile: { username: 'Bot', expectedUuid: 'uuid', actualUuid: 'uuid' },
    uuid: 'uuid',
    timeline: [{ name: 'login', at: 1, summary: [] }],
    packetTrace: [{ name: 'success', state: 'login', keys: ['uuid'] }],
    serverLogs: [{ stream: 'stdout', text: `loaded ${root} on 25565\n` }],
    error: new Error('spawn timeout'),
    collectArtifacts: async () => ({
      events: [{ name: 'login', at: 1, summary: [] }],
      logs: [{ stream: 'stdout', text: `loaded ${root} on 25565\n` }],
      serverProperties: 'server-port=25565\n',
      eula: 'eula=true\n'
    })
  }

  try {
    const bundle = await writeLoginDebugBundle({
      outputRoot,
      name: 'bad login',
      session,
      parityDiff: [{ path: 'events', official: ['spawn'], rebuilt: ['kicked'] }]
    })
    const manifest = JSON.parse(await readFile(bundle.manifest, 'utf8'))
    assert.equal(path.basename(bundle.bundleDir), 'bad-login')
    assert.equal(manifest.profile.username, 'Bot')
    assert.equal(manifest.error.message, 'spawn timeout')
    assert.deepEqual(JSON.parse(await readFile(path.join(bundle.bundleDir, 'packet-trace.json'), 'utf8')), session.packetTrace)
    assert.match(await readFile(path.join(bundle.bundleDir, 'server.log'), 'utf8'), /loaded/)
    assert.match(await readFile(path.join(bundle.bundleDir, 'normalized-artifacts.json'), 'utf8'), /<run-dir>/)
    assert.match(await readFile(path.join(bundle.bundleDir, 'parity-diff.txt'), 'utf8'), /VibeCraft/)
    assert.equal(await readFile(path.join(bundle.bundleDir, 'world', 'level.dat'), 'utf8'), 'fake')
    assert.equal(await readFile(path.join(bundle.bundleDir, 'server.properties'), 'utf8'), 'server-port=25565\n')
  } finally {
    await rm(root, { recursive: true, force: true })
    await rm(outputRoot, { recursive: true, force: true })
  }
})

test('writeLoginDebugBundle can skip temp world copying for smaller artifacts', async () => {
  const outputRoot = await createTempWorld('vibecraft-mf-debug-skip-')
  try {
    const bundle = await writeLoginDebugBundle({
      outputRoot,
      name: 'no-world',
      includeWorld: false,
      session: {
        profile: { username: 'Bot' },
        timeline: [],
        packetTrace: [],
        serverLogs: []
      }
    })
    const manifest = JSON.parse(await readFile(bundle.manifest, 'utf8'))
    assert.equal(manifest.profile.username, 'Bot')
    assert.equal(await readFile(path.join(bundle.bundleDir, 'parity-diff.txt'), 'utf8'), 'official and VibeCraft artifacts match\n')
  } finally {
    await rm(outputRoot, { recursive: true, force: true })
  }
})
