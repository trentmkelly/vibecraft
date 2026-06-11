import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
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

test('raw 26.1.2 parallel offline logins isolate profile, configuration, play, and keepalive state', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-parallel-login-')
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
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const suffix = crypto.randomUUID().replaceAll('-', '').slice(0, 6)
    const usernames = ['A', 'B', 'C', 'D'].map(name => `Par${name}${suffix}`)
    const started = Promise.all(usernames.map(username => runJoinProbe(port, username)))
    const results = await started

    assert.equal(results.length, usernames.length)
    assert.deepEqual(results.map(result => result.ok), usernames.map(() => true))

    const uuids = new Set()
    for (const [index, result] of results.entries()) {
      const username = usernames[index]
      assert.equal(result.joinState.profile.name, username)
      assert.equal(result.joinState.profile.uuid, offlineUuid(username))
      assert.ok(result.joinState.entityId > 0, `${username} should receive a positive entity id`)
      assert.equal(result.joinState.dimension, 'minecraft:overworld')
      assert.ok(result.joinState.initialChunkCount > 0, `${username} should receive an initial chunk batch`)
      assert.equal(result.joinState.lastReceivedChunk, result.joinState.initialChunkCount - 1)
      assert.ok(result.config.some(packet => packet.id === 3), `${username} should finish configuration`)
      assert.ok(result.play.some(packet => packet.id === 49), `${username} should reach play login`)
      assert.ok(result.play.some(packet => packet.id === 70), `${username} should receive its own tab-list profile`)
      assert.ok(result.keepAliveReplies >= 1, `${username} should keep its own keepalive response state`)
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
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '17000'
      },
      timeout: 45_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

function offlineUuid (name) {
  const hash = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest()
  hash[6] = (hash[6] & 0x0f) | 0x30
  hash[8] = (hash[8] & 0x3f) | 0x80
  const hex = hash.toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
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
