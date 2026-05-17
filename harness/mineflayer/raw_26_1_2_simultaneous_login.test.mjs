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

test('raw 26.1.2 simultaneous offline logins isolate generated profiles', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-simultaneous-login-')
  const usernames = ['SimulA', 'SimulB', 'SimulC', 'SimulD']
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const results = await Promise.all(usernames.map(username => runJoinProbe(port, username)))
    const uuids = new Set()

    for (const [index, result] of results.entries()) {
      const username = usernames[index]
      assert.equal(result.ok, true)
      assert.equal(result.joinState.profile.name, username)
      assert.equal(result.joinState.profile.uuid, offlineUuid(username))
      assert.equal(result.joinState.dimension, 'minecraft:overworld')
      assert.equal(result.joinState.lastReceivedChunk, 8)
      assert.ok(result.config.some(packet => packet.id === 3), `${username} should finish configuration`)
      assert.ok(result.play.some(packet => packet.id === 49), `${username} should receive play login`)
      assert.ok(result.play.some(packet => packet.id === 70), `${username} should receive a tab-list profile`)
      uuids.add(result.joinState.profile.uuid)
    }

    assert.equal(uuids.size, usernames.length)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

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
