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
      RUSTCRAFT_EXPECT_IS_FLAT: 'false',
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify({ x: 0.5, y: 112, z: 0.5, yaw: 0, pitch: 0 }),
      RUSTCRAFT_EXPECT_DEFAULT_SPAWN: JSON.stringify({ x: 0, y: 112, z: 0 })
    })

    assert.equal(joined.ok, true)
    assert.deepEqual(joined.joinState.loginDistances, {
      viewDistance: 5,
      simulationDistance: 3
    })
    assert.deepEqual({
      cacheCenter: joined.joinState.chunkStreaming.cacheCenter,
      cacheRadius: joined.joinState.chunkStreaming.cacheRadius,
      batchSize: joined.joinState.chunkStreaming.batchSize,
      chunks: joined.joinState.chunkStreaming.chunks
    }, {
      cacheCenter: {
        x: 0,
        z: 0
      },
      cacheRadius: 5,
      batchSize: 121,
      chunks: expectedChunkSquare(5)
    })
    assert.equal(joined.joinState.initialChunkCount, 121)
    assert.equal(joined.joinState.chunkStreaming.chunkBiomePalettes.length, 121)
    assert.ok(joined.joinState.chunkStreaming.chunkBiomePalettes.every(chunk => chunk.sectionCount >= 1))
    assert.ok(joined.joinState.chunkStreaming.chunkBiomePalettes.every(chunk =>
      chunk.biomePalette.length >= 1 &&
      chunk.biomePalette.every(biome => biome.startsWith('minecraft:'))
    ))
    assert.ok(!joined.joinState.chunkStreaming.chunkBiomePalettes.some(chunk =>
      chunk.biomePalette.includes('minecraft:badlands')
    ))
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

function expectedChunkSquare (radius) {
  const chunks = []
  for (let z = -radius; z <= radius; z += 1) {
    for (let x = -radius; x <= radius; x += 1) {
      chunks.push({ x, z })
    }
  }
  return chunks
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
