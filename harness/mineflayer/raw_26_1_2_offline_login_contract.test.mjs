import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { readFile, rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

import {
  createTempWorld,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 offline login reaches play without auth, encryption, or profile-key requirements', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-offline-contract-')
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

    const login = await runJoinProbe(port)
    assert.equal(login.ok, true)
    assert.equal(login.login, 2, 'offline login should receive login_finished, not encryption hello')
    assert.ok(login.config.some(packet => packet.id === 3), 'expected finish configuration')
    assert.ok(login.play.some(packet => packet.id === 49), 'expected play login')
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

test('minimal offline login path does not call session services or require secure profile keys', async () => {
  const statusSource = await readFile(path.join(repoRoot, 'src', 'network', 'status.rs'), 'utf8')
  const statusLoginSource = await readFile(path.join(repoRoot, 'src', 'network', 'status', 'chunk_a.rs'), 'utf8')
  const loginSource = await readFile(path.join(repoRoot, 'src', 'network', 'login.rs'), 'utf8')

  assert.match(statusLoginSource, /accept_offline_hello/)
  assert.doesNotMatch(statusSource, /start_encryption\(/)
  assert.doesNotMatch(statusLoginSource, /start_encryption\(/)
  assert.doesNotMatch(statusSource, /sessionserver|Yggdrasil|profile[_-]?key/i)
  assert.doesNotMatch(statusLoginSource, /sessionserver|Yggdrasil|profile[_-]?key/i)
  assert.match(loginSource, /accept_offline_hello/)
})

async function runJoinProbe (port) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: 'OfflineContract'
      },
      timeout: 30000,
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
