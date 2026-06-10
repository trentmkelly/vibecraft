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

test('raw 26.1.2 login visibility starts only after configuration enters play', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-login-visibility-')
  const username = 'VisibleRaw'
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

    const joined = await runJoinProbe(port, username)

    assert.equal(joined.ok, true)
    assert.ok(joined.config.some(packet => packet.id === 14), 'configuration should finish before play visibility')
    assert.ok(joined.config.every(packet => packet.id !== 70), 'tab-list visibility must not be sent during configuration')
    assert.ok(joined.config.every(packet => packet.id !== 49), 'play login must not be sent during configuration')
    assert.equal(joined.play[0].id, 49, 'play login should be the first play packet')
    assert.ok(joined.play.some(packet => packet.id === 70), 'play state should add the player to the tab list')
    assert.equal(joined.joinState.profile.name, username)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
    assert.ok(joined.joinState.initialChunkCount > 0, 'play entry should receive an initial chunk batch')
    assert.equal(joined.joinState.lastReceivedChunk, joined.joinState.initialChunkCount - 1)
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
        VIBECRAFT_USERNAME: username
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
