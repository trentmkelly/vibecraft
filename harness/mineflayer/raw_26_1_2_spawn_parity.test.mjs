import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'

test('raw 26.1.2 first-login spawn packets advertise seed, flatness, spawn, and initial chunks', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-spawn-parity-')
  const seed = 1234567890123n
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed,
      properties: {
        'view-distance': '3',
        'simulation-distance': '4',
        'spawn-protection': '16'
      }
    })
    server = await start(root, port)

    const joined = await runJoinProbe(port, 'SpawnParity', {
      VIBECRAFT_EXPECT_WORLD_SEED: seed.toString(),
      VIBECRAFT_EXPECT_IS_FLAT: 'false',
      VIBECRAFT_EXPECT_JOIN_POSITION: JSON.stringify({ x: -32.5, y: 76, z: 10.5, yaw: 0, pitch: 0 })
    })

    assert.equal(joined.ok, true)
    assert.deepEqual(joined.joinState.defaultSpawn, {
      dimension: 'minecraft:overworld',
      x: -31,
      y: 65,
      z: 6
    })
    assert.equal(joined.joinState.loginSpawnInfo.seed, seed.toString())
    assert.equal(joined.joinState.loginSpawnInfo.isFlat, false)
    assert.equal(joined.joinState.loginSpawnInfo.seaLevel, 63)
    assert.ok(joined.joinState.initialChunkCount > 0, 'spawn login should receive an initial chunk batch')
    assert.equal(joined.joinState.lastReceivedChunk, joined.joinState.initialChunkCount - 1)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function start (root, port) {
  const server = startVibeCraft({ binary, root, port, levelName: 'world' })
  await waitForPort(port, host, 10_000)
  return server
}

async function runJoinProbe (port, username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        ...env
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
    server.listen(0, host, resolve)
    server.once('error', reject)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
