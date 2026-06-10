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

// Item protocol ids (official 26.1.2 registries.json).
const OAK_STAIRS_ITEM_ID = 442
const SAND_ITEM_ID = 59
// Block-state network ids (official 26.1.2 blocks.json, vendored at
// vanilla-data/reports/blocks_26_1_2.json).
const OAK_STAIRS_EAST_BOTTOM_STRAIGHT_DRY = 3978
const SAND_STATE_ID = 118
const FALLING_BLOCK_ENTITY_TYPE = 51

test('raw 26.1.2 creative placement orients stairs from the player facing', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-block-place-live-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        gamemode: 'creative',
        'spawn-protection': '0',
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const joined = await runJoinProbe(port, 'PlaceBot', {
      VIBECRAFT_EXPECT_GAME_MODE: '1',
      VIBECRAFT_EXPECT_ABILITY_FLAGS: '13',
      VIBECRAFT_PLACEMENT_PROBE: JSON.stringify({
        itemId: OAK_STAIRS_ITEM_ID,
        // Stand above the terrain and click an air cell within reach: a
        // replaceable clicked block places in-place (BlockPlaceContext).
        playerX: 0.5,
        playerY: 120,
        playerZ: 0.5,
        yaw: -90, // facing east -> Java getHorizontalDirection() = east
        pitch: 45,
        x: 0,
        y: 118,
        z: 0,
        face: 1, // up
        cursorY: 1.0,
        sequence: 7,
        captureMs: 2500
      })
    })

    assert.equal(joined.ok, true)
    assert.ok(joined.placement, 'placement probe results missing')
    assert.ok(
      joined.placement.ackSequences.includes(7),
      `block_changed_ack for sequence 7 missing: ${JSON.stringify(joined.placement.ackSequences)}`
    )
    const placed = joined.placement.blockUpdates.find(
      update => update.x === 0 && update.y === 118 && update.z === 0
    )
    assert.ok(placed, `no block update at the placement target: ${JSON.stringify(joined.placement.blockUpdates)}`)
    assert.equal(
      placed.stateId,
      OAK_STAIRS_EAST_BOTTOM_STRAIGHT_DRY,
      'stairs should place facing east (player yaw -90), bottom half, straight, dry'
    )
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

test('raw 26.1.2 placed sand falls as a falling-block entity and lands', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-sand-gravity-live-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        gamemode: 'creative',
        'spawn-protection': '0',
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const joined = await runJoinProbe(port, 'SandBot', {
      VIBECRAFT_EXPECT_GAME_MODE: '1',
      VIBECRAFT_EXPECT_ABILITY_FLAGS: '13',
      VIBECRAFT_PLACEMENT_PROBE: JSON.stringify({
        itemId: SAND_ITEM_ID,
        playerX: 0.5,
        playerY: 120,
        playerZ: 0.5,
        yaw: 0,
        pitch: 45,
        x: 0,
        y: 118,
        z: 0,
        face: 1,
        cursorY: 1.0,
        sequence: 9,
        // Long enough for the 2-tick fall delay plus the descent and landing.
        captureMs: 5000
      })
    })

    assert.equal(joined.ok, true)
    const placement = joined.placement
    assert.ok(placement, 'placement probe results missing')
    assert.ok(placement.ackSequences.includes(9), 'block_changed_ack missing')

    // The sand block appears at the target, then is replaced by air when the
    // falling entity spawns.
    const placedSand = placement.blockUpdates.find(
      update => update.y === 118 && update.stateId === SAND_STATE_ID
    )
    assert.ok(placedSand, `sand never placed: ${JSON.stringify(placement.blockUpdates)}`)
    const clearedToAir = placement.blockUpdates.find(
      update => update.y === 118 && update.stateId === 0
    )
    assert.ok(clearedToAir, 'sand cell should clear to air when the fall starts')

    const fallingSpawn = placement.addedEntities.find(
      entity => entity.type === FALLING_BLOCK_ENTITY_TYPE
    )
    assert.ok(fallingSpawn, `no falling_block entity spawned: ${JSON.stringify(placement.addedEntities)}`)
    assert.ok(
      placement.removedEntityIds.includes(fallingSpawn.entityId),
      'falling-block entity should be removed on landing'
    )

    // The landing writes sand somewhere below the spawn cell.
    const landed = placement.blockUpdates.find(
      update => update.stateId === SAND_STATE_ID && update.y < 118
    )
    assert.ok(landed, `sand never landed below: ${JSON.stringify(placement.blockUpdates)}`)
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
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        ...env
      },
      timeout: 45_000,
      maxBuffer: 4 * 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
