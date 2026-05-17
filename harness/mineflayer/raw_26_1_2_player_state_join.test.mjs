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

test('raw 26.1.2 survival join synchronizes baseline player state packets', { timeout: 90_000 }, async () => {
  const result = await withServer('rustcraft-player-state-survival-', {
    seed: 112233,
    properties: {
      gamemode: 'survival'
    }
  }, port => runJoinProbe(port, 'StateSurvival', {
    RUSTCRAFT_EXPECT_WORLD_SEED: '112233',
    RUSTCRAFT_EXPECT_GAME_MODE: '0',
    RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
    RUSTCRAFT_EXPECT_ABILITY_FLAGS: '0',
    RUSTCRAFT_EXPECT_HEALTH: '20',
    RUSTCRAFT_EXPECT_FOOD_LEVEL: '20',
    RUSTCRAFT_EXPECT_FOOD_SATURATION: '5',
    RUSTCRAFT_EXPECT_XP_PROGRESS: '0',
    RUSTCRAFT_EXPECT_XP_LEVEL: '0',
    RUSTCRAFT_EXPECT_XP_TOTAL: '0'
  }))

  assert.equal(result.ok, true)
  assert.equal(result.joinState.loginSpawnInfo.gameMode, 0)
  assert.deepEqual(result.joinState.position, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
})

test('raw 26.1.2 creative join synchronizes gamemode and ability flags', { timeout: 90_000 }, async () => {
  const result = await withServer('rustcraft-player-state-creative-', {
    seed: 445566,
    properties: {
      gamemode: 'creative',
      'force-gamemode': 'true'
    }
  }, port => runJoinProbe(port, 'StateCreative', {
    RUSTCRAFT_EXPECT_WORLD_SEED: '445566',
    RUSTCRAFT_EXPECT_GAME_MODE: '1',
    RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
    RUSTCRAFT_EXPECT_ABILITY_FLAGS: '13',
    RUSTCRAFT_EXPECT_HEALTH: '20',
    RUSTCRAFT_EXPECT_FOOD_LEVEL: '20',
    RUSTCRAFT_EXPECT_FOOD_SATURATION: '5',
    RUSTCRAFT_EXPECT_XP_PROGRESS: '0',
    RUSTCRAFT_EXPECT_XP_LEVEL: '0',
    RUSTCRAFT_EXPECT_XP_TOTAL: '0'
  }))

  assert.equal(result.ok, true)
  assert.equal(result.joinState.loginSpawnInfo.gameMode, 1)
})

async function withServer (prefix, options, callback) {
  const port = await reservePort()
  const root = await createTempWorld(prefix)
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: options.seed,
      properties: {
        'view-distance': '4',
        'simulation-distance': '4',
        ...(options.properties ?? {})
      }
    })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
    return await callback(port)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
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
