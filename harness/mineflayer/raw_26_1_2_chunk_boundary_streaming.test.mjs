import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
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

test('raw 26.1.2 movement across a chunk boundary streams terrain around the new center', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-chunk-boundary-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: 97531,
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const moved = await runJoinProbe(port, 'ChunkWalker', {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify({ x: 32.5, y: 80, z: -16.5, yaw: 0, pitch: 0 }),
      RUSTCRAFT_RAW_PROBE_POST_ACTION_MS: '1000'
    })

    assert.equal(moved.ok, true)
    assert.deepEqual(moved.joinState.dynamicChunkStreaming, {
      cacheCenter: {
        x: 2,
        z: -2
      },
      batchSize: 9,
      chunks: [
        { x: 1, z: -3 },
        { x: 2, z: -3 },
        { x: 3, z: -3 },
        { x: 1, z: -2 },
        { x: 2, z: -2 },
        { x: 3, z: -2 },
        { x: 1, z: -1 },
        { x: 2, z: -1 },
        { x: 3, z: -1 }
      ],
      forgottenChunks: [
        { x: -1, z: -1 },
        { x: -1, z: 0 },
        { x: -1, z: 1 },
        { x: 0, z: -1 },
        { x: 0, z: 0 },
        { x: 0, z: 1 },
        { x: 1, z: 0 },
        { x: 1, z: 1 }
      ]
    })
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

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
