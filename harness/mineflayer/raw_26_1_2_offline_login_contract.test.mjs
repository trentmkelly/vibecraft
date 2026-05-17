import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

test('raw 26.1.2 offline login reaches play without auth, encryption, or profile-key requirements', async () => {
  const login = await runJoinProbe()
  assert.equal(login.ok, true)
  assert.equal(login.login, 2, 'offline login should receive login_finished, not encryption hello')
  assert.ok(login.config.some(packet => packet.id === 3), 'expected finish configuration')
  assert.ok(login.play.some(packet => packet.id === 49), 'expected play login')
})

test('minimal offline login path does not call session services or require secure profile keys', async () => {
  const statusSource = await readFile(path.join(repoRoot, 'src', 'network', 'status.rs'), 'utf8')
  const loginSource = await readFile(path.join(repoRoot, 'src', 'network', 'login.rs'), 'utf8')

  assert.match(statusSource, /accept_offline_hello/)
  assert.doesNotMatch(statusSource, /start_encryption\(/)
  assert.doesNotMatch(statusSource, /sessionserver|Yggdrasil|profile[_-]?key/i)
  assert.match(loginSource, /accept_offline_hello/)
})

async function runJoinProbe () {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: 'OfflineContract'
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
