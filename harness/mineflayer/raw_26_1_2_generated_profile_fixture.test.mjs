import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { readFile, rm } from 'node:fs/promises'
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

test('raw 26.1.2 generated profiles write exact offline UUIDs to usercache', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-generated-profile-')
  const usernames = ['GenProfileA', 'GenProfileB']
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
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, '127.0.0.1', 10_000)

    for (const username of usernames) {
      assert.equal(offlineUuid(username), await expectedBeforeConnect(username))
      const login = await runJoinProbe(port, username)
      assert.equal(login.ok, true)
      assert.equal(login.joinState.profile.uuid, offlineUuid(username))
    }

    const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
    for (const username of usernames) {
      const entry = usercache.find(candidate => candidate.name === username)
      assert.ok(entry, `${username} should be cached after login`)
      assert.equal(entry.uuid, offlineUuid(username), `${username} cache UUID should match offline derivation`)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function expectedBeforeConnect (username) {
  return offlineUuid(username)
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
