import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
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

test('raw 26.1.2 login storm isolates generated profiles across temp worlds and ports', { timeout: 90_000 }, async () => {
  const worlds = await Promise.all([0, 1, 2].map(startWorld))

  try {
    const probes = worlds.flatMap((world, worldIndex) => [0, 1].map(profileIndex => ({
      port: world.port,
      username: `Storm${worldIndex}${profileIndex}`
    })))
    const results = await Promise.all(probes.map(probe => runJoinProbe(probe.port, probe.username)))
    const accepted = results.filter(result => result.ok)
    const uuids = new Set()

    assert.equal(accepted.length, probes.length)

    for (const [index, result] of results.entries()) {
      const { username } = probes[index]
      assert.equal(result.joinState.profile.name, username)
      assert.equal(result.joinState.profile.uuid, offlineUuid(username))
      assert.equal(result.joinState.dimension, 'minecraft:overworld')
      assert.equal(result.joinState.lastReceivedChunk, 8)
      assert.ok(result.config.some(packet => packet.id === 3), `${username} should finish configuration`)
      assert.ok(result.play.some(packet => packet.id === 49), `${username} should reach play login`)
      assert.ok(result.play.some(packet => packet.id === 70), `${username} should receive tab-list identity`)
      uuids.add(result.joinState.profile.uuid)
    }

    assert.equal(uuids.size, probes.length)
  } finally {
    await Promise.all(worlds.map(async world => {
      await stopServer(world.server.child)
      await rm(world.root, { recursive: true, force: true })
    }))
  }
})

async function startWorld (index) {
  const port = await reservePort()
  const root = await createTempWorld(`rustcraft-login-storm-${index}-`)
  await writeOfflineServerFiles(root, {
    port,
    levelName: 'world',
    properties: {
      'view-distance': '4',
      'simulation-distance': '4'
    }
  })
  const server = startRustCraft({ binary, root, port, levelName: 'world' })
  await waitForPort(port, host, 10_000)
  return { port, root, server }
}

async function runJoinProbe (port, username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username
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
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
