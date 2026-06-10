import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
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

test('raw 26.1.2 reconnect after ban and pardon preserves stale-session cleanup', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-reconnect-ban-pardon-')
  const username = 'KickFallback'
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

    const first = await runJoinProbe(port, username)
    assert.equal(first.ok, true)
    assert.equal(first.joinState.profile.uuid, offlineUuid(username))

    await writeFile(
      path.join(root, 'banned-players.json'),
      `${JSON.stringify([{ ...profile(username), created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'reconnect fallback' }])}\n`
    )
    server.child.stdin.write('reload\n')
    await delay(250)

    const banned = await runJoinProbe(port, username, { VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(banned.reason, /multiplayer\.disconnect\.banned/)

    await writeFile(path.join(root, 'banned-players.json'), '[]\n')
    server.child.stdin.write('reload\n')
    await delay(250)

    const rejoined = await runJoinProbe(port, username, {
      VIBECRAFT_RAW_PROBE_KEEPALIVE_MS: '12000'
    })
    assert.equal(rejoined.ok, true)
    assert.equal(rejoined.joinState.profile.uuid, offlineUuid(username))
    assert.ok(rejoined.config.some(packet => packet.id === 3))
    assert.ok(rejoined.play.some(packet => packet.id === 49))
    assert.ok(rejoined.play.some(packet => packet.id === 70))
    assert.equal(rejoined.joinState.lastReceivedChunk, 8)
    assert.ok(rejoined.keepAliveReplies > 0)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

function profile (name) {
  return { uuid: offlineUuid(name), name }
}

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

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
