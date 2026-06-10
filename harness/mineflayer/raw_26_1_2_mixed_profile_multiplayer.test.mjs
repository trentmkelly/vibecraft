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

test('raw 26.1.2 mixed offline profiles preserve access decisions and duplicate cleanup', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-mixed-profile-')
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
    await writeAccessFiles(root)
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const accepted = await Promise.all([
      runJoinProbe(port, 'MixListed'),
      runJoinProbe(port, 'MixOp')
    ])
    for (const joined of accepted) {
      assert.equal(joined.ok, true)
      assert.equal(joined.joinState.profile.uuid, offlineUuid(joined.joinState.profile.name))
      assert.ok(joined.config.some(packet => packet.id === 3))
      assert.ok(joined.play.some(packet => packet.id === 49))
      assert.ok(joined.play.some(packet => packet.id === 70))
      assert.equal(joined.joinState.lastReceivedChunk, 8)
    }

    const banned = await runJoinProbe(port, 'MixBanned', { VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(banned.reason, /multiplayer\.disconnect\.banned/)

    await runDuplicateProbe(port, 'MixDup')
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

function profile (name) {
  return { uuid: offlineUuid(name), name }
}

async function writeAccessFiles (root) {
  await Promise.all([
    writeFile(path.join(root, 'whitelist.json'), `${JSON.stringify([profile('MixListed')])}\n`),
    writeFile(path.join(root, 'ops.json'), `${JSON.stringify([{ ...profile('MixOp'), level: 4, bypassesPlayerLimit: false }])}\n`),
    writeFile(path.join(root, 'banned-players.json'), `${JSON.stringify([{ ...profile('MixBanned'), created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'mixed profile test' }])}\n`)
  ])
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

async function runDuplicateProbe (port, username) {
  await execFileAsync(
    process.execPath,
    ['raw_26_1_2_duplicate_login.test.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_DUPLICATE_USERNAME: username
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )
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
