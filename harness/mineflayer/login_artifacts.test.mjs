import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdir, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { createTempWorld, offlineUuid } from './runner.mjs'
import {
  collectOfflineLoginArtifact,
  collectOfflineLoginArtifactPair,
  diffNormalizedLogs
} from './login_artifacts.mjs'

test('collectOfflineLoginArtifact captures baseline files, UUIDs, playerdata paths, and normalized logs', async () => {
  const root = await createTempWorld('vibecraft-login-artifact-')
  const session = await fakeLoginSession(root, {
    role: 'vibecraft',
    username: 'ArtifactBot',
    port: 25565,
    logLine: `2026-05-17T12:00:00Z ${root} accepted ArtifactBot on 25565\n`
  })

  try {
    const artifact = await collectOfflineLoginArtifact(session)
    assert.equal(artifact.role, 'vibecraft')
    assert.equal(artifact.profile.username, 'ArtifactBot')
    assert.equal(artifact.uuid, offlineUuid('ArtifactBot'))
    assert.equal(artifact.botUuid, offlineUuid('ArtifactBot'))
    assert.match(artifact.files.serverProperties, /^online-mode=false$/m)
    assert.equal(artifact.files.eula, 'eula=true\n')
    assert.deepEqual(artifact.files.usercache, [{
      name: 'ArtifactBot',
      uuid: offlineUuid('ArtifactBot')
    }])
    assert.deepEqual(artifact.files.playerdataPaths, [
      `world/playerdata/${offlineUuid('ArtifactBot')}.dat`
    ])
    assert.equal(
      artifact.normalized.logs[0].text,
      '<timestamp> <run-dir> accepted <username> on <port>\n'
    )
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

test('collectOfflineLoginArtifactPair reports normalized official-vs-VibeCraft log diffs', async () => {
  const root = await createTempWorld('vibecraft-login-artifact-pair-')
  const officialRoot = path.join(root, 'official')
  const vibecraftRoot = path.join(root, 'vibecraft')
  const official = await fakeLoginSession(officialRoot, {
    role: 'official',
    username: 'PairBot',
    port: 25565,
    logLine: `2026-05-17T12:00:00Z ${officialRoot} accepted PairBot on 25565\n`
  })
  const vibecraft = await fakeLoginSession(vibecraftRoot, {
    role: 'vibecraft',
    username: 'PairBot',
    port: 25566,
    logLine: `2026-05-17T12:00:01Z ${vibecraftRoot} kicked PairBot on 25566\n`
  })

  try {
    const pair = await collectOfflineLoginArtifactPair(official, vibecraft)
    assert.equal(pair.official.role, 'official')
    assert.equal(pair.vibecraft.role, 'vibecraft')
    assert.deepEqual(pair.normalizedLogDiff, [{
      official: [{ stream: 'stdout', text: '<timestamp> <run-dir> accepted <username> on <port>\n' }],
      path: 'logs',
      rebuilt: [{ stream: 'stdout', text: '<timestamp> <run-dir> kicked <username> on <port>\n' }]
    }])
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

test('diffNormalizedLogs returns no diff for volatile-only official and VibeCraft log differences', async () => {
  const official = {
    root: '/tmp/official',
    endpoint: { port: 25565 },
    profile: { username: 'StableBot' },
    normalized: {
      logs: [{ stream: 'stdout', text: '<timestamp> <run-dir> accepted <username> on <port>\n' }]
    }
  }
  const vibecraft = {
    root: '/tmp/vibecraft',
    endpoint: { port: 25566 },
    profile: { username: 'StableBot' },
    normalized: {
      logs: [{ stream: 'stdout', text: '<timestamp> <run-dir> accepted <username> on <port>\n' }]
    }
  }
  assert.deepEqual(diffNormalizedLogs(official, vibecraft), [])
})

async function fakeLoginSession(root, options) {
  const uuid = offlineUuid(options.username)
  const world = path.join(root, 'world')
  const playerdata = path.join(world, 'playerdata')
  await mkdir(playerdata, { recursive: true })
  await writeFile(path.join(root, 'server.properties'), [
    'online-mode=false',
    `server-port=${options.port}`,
    'level-name=world',
    ''
  ].join('\n'))
  await writeFile(path.join(root, 'eula.txt'), 'eula=true\n')
  await writeFile(path.join(root, 'usercache.json'), `${JSON.stringify([{ name: options.username, uuid }])}\n`)
  await writeFile(path.join(playerdata, `${uuid}.dat`), 'playerdata')
  return {
    root,
    paths: {
      root,
      world,
      serverProperties: path.join(root, 'server.properties'),
      eula: path.join(root, 'eula.txt')
    },
    endpoint: { port: options.port },
    profile: {
      username: options.username,
      expectedUuid: uuid,
      actualUuid: uuid
    },
    uuid,
    serverLogs: [{ stream: 'stdout', text: options.logLine }]
  }
}
