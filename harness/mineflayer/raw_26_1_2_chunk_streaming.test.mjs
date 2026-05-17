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

test('raw 26.1.2 initial chunk stream preserves vanilla readiness ordering and distances', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-chunk-stream-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: 24680,
      properties: {
        'view-distance': '5',
        'simulation-distance': '3'
      }
    })
    server = await start(root, port)

    const joined = await runJoinProbe(port, 'ChunkStream', {
      RUSTCRAFT_EXPECT_WORLD_SEED: '24680',
      RUSTCRAFT_EXPECT_IS_FLAT: 'true'
    })

    assert.equal(joined.ok, true)
    assert.deepEqual(joined.joinState.loginDistances, {
      viewDistance: 5,
      simulationDistance: 3
    })
    assert.deepEqual(joined.joinState.chunkStreaming, {
      cacheRadius: 5,
      batchSize: 9,
      chunks: [
        { x: -1, z: -1 },
        { x: 0, z: -1 },
        { x: 1, z: -1 },
        { x: -1, z: 0 },
        { x: 0, z: 0 },
        { x: 1, z: 0 },
        { x: -1, z: 1 },
        { x: 0, z: 1 },
        { x: 1, z: 1 }
      ]
    })
    assert.equal(joined.joinState.initialChunkCount, 9)
    assert.equal(joined.play.at(-1).id, 11)
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
