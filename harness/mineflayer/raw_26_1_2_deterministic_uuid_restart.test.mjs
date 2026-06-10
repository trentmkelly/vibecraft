import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')

test('raw 26.1.2 offline UUIDs are deterministic across restart and case variants', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-uuid-restart-')
  const usernames = ['UuidAlpha', 'UuidBravo', 'UuidCase', 'uuidcase']
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '2',
        'simulation-distance': '2'
      }
    })
    server = await startTempServer(root, port)
    const first = await joinAll(port, usernames)
    await stopServer(server.child)
    server = null

    server = await startTempServer(root, port)
    const second = await joinAll(port, usernames)

    for (const username of usernames) {
      assert.equal(first.get(username), offlineUuid(username), `${username} first UUID should be offline-mode derived`)
      assert.equal(second.get(username), first.get(username), `${username} UUID should survive restart`)
    }
    assert.notEqual(
      first.get('UuidCase'),
      first.get('uuidcase'),
      'offline UUID derivation must stay case-sensitive across restart'
    )
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function startTempServer (root, port) {
  const server = startVibeCraft({ binary, root, port, levelName: 'world' })
  await waitForPort(port, '127.0.0.1', 10_000)
  return server
}

async function joinAll (port, usernames) {
  const results = new Map()
  for (const username of usernames) {
    const login = await runJoinProbe(port, username)
    assert.equal(login.ok, true, `${username} should reach play`)
    assert.equal(login.joinState.profile.name, username)
    results.set(username, login.joinState.profile.uuid)
  }
  return results
}

async function runJoinProbe (port, username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_EXPECT_WORLD_SEED: '8675309',
        VIBECRAFT_EXPECT_IS_FLAT: 'false',
        VIBECRAFT_EXPECT_JOIN_POSITION: JSON.stringify({ x: 0.5, y: 112, z: 0.5, yaw: 0, pitch: 0 }),
        VIBECRAFT_EXPECT_DEFAULT_SPAWN: JSON.stringify({ x: 0, y: 112, z: 0 })
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, '127.0.0.1', resolve)
  })
  const { port } = server.address()
  await new Promise((resolve, reject) => {
    server.close(error => error ? reject(error) : resolve())
  })
  return port
}
