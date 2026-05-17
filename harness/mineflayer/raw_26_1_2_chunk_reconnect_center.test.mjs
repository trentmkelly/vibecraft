import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 reconnect streams chunks around saved player chunk', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-chunk-reconnect-')
  const username = 'ChunkReturn'
  const uuid = offlineUuid(username)
  const savedPosition = { x: 40.5, y: 80, z: -40.5, yaw: 180, pitch: 0 }
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: 13579,
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = await start(root, port)

    const moved = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(savedPosition)
    })
    assert.equal(moved.ok, true)

    await waitForFile(path.join(root, 'world', 'playerdata', `${uuid}.dat`))
    await stopServer(server.child)
    server = await start(root, port)

    const rejoined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_WORLD_SEED: '13579',
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(savedPosition)
    })
    assert.equal(rejoined.ok, true)
    assert.deepEqual(rejoined.joinState.chunkStreaming, {
      cacheCenter: {
        x: 2,
        z: -3
      },
      cacheRadius: 4,
      batchSize: 9,
      chunks: [
        { x: 1, z: -4 },
        { x: 2, z: -4 },
        { x: 3, z: -4 },
        { x: 1, z: -3 },
        { x: 2, z: -3 },
        { x: 3, z: -3 },
        { x: 1, z: -2 },
        { x: 2, z: -2 },
        { x: 3, z: -2 }
      ]
    })
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function start (root, port) {
  const server = startRustCraft({ binary, root, port, levelName: 'world' })
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
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username,
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

async function waitForFile (file) {
  const deadline = Date.now() + 5_000
  while (Date.now() < deadline) {
    try {
      await access(file)
      return
    } catch {
      await delay(50)
    }
  }
  throw new Error(`timed out waiting for ${file}`)
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
