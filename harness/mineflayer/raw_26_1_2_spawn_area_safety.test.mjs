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
const seeds = [0n, 1n, -1n, 8675309n, 987654321987n]

test('raw 26.1.2 seeded spawn areas are collision-safe and immediately movable', { timeout: 180_000 }, async () => {
  for (const [index, seed] of seeds.entries()) {
    const port = await reservePort()
    const seedLabel = seed.toString().replace('-', 'neg')
    const root = await createTempWorld(`rustcraft-spawn-area-${seedLabel}-`)
    let server

    try {
      await writeOfflineServerFiles(root, {
        port,
        levelName: 'world',
        seed,
        properties: {
          'view-distance': '3',
          'simulation-distance': '4'
        }
      })
      server = await start(root, port)

      const joined = await runJoinProbe(port, `Spawn${index}`, {
        RUSTCRAFT_EXPECT_WORLD_SEED: seed.toString(),
        RUSTCRAFT_EXPECT_IS_FLAT: 'true',
        RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
        RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify({ x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 }),
        RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
      })

      assert.equal(joined.ok, true, `seed ${seed} should join cleanly`)
      assert.equal(joined.aborted, true, `seed ${seed} should reach first-tick movement`)
      assert.deepEqual(joined.joinState.position, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
      assert.deepEqual(joined.joinState.defaultSpawn, {
        dimension: 'minecraft:overworld',
        x: 0,
        y: 80,
        z: 0
      })
      assert.equal(joined.joinState.initialChunkCount, 9)
    } finally {
      if (server) await stopServer(server.child)
      await rm(root, { recursive: true, force: true })
    }
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
