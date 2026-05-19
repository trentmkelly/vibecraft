import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const username = process.env.RUSTCRAFT_RETURNING_USERNAME ?? `Ret${crypto.randomUUID().replaceAll('-', '').slice(0, 10)}`

test('raw 26.1.2 returning offline login reuses the same profile identity', { timeout: 45_000 }, async () => {
  const first = await runJoinProbe(username)
  assert.equal(first.ok, true)
  assert.equal(first.joinState.profile.name, username)
  assert.equal(first.joinState.profile.uuid, offlineUuid(username))
  assert.equal(first.joinState.entityId, 1)
  assert.equal(first.joinState.position.x, 0.5)
  assert.ok(first.joinState.position.y > -64)
  assert.equal(first.joinState.position.z, 0.5)
  assert.equal(first.joinState.position.yaw, 0)
  assert.equal(first.joinState.position.pitch, 0)

  const second = await runJoinProbe(username)
  assert.equal(second.ok, true)
  assert.equal(second.joinState.profile.name, username)
  assert.equal(second.joinState.profile.uuid, first.joinState.profile.uuid)
  assert.equal(second.compressionThreshold, first.compressionThreshold)
  assert.equal(second.joinState.entityId, first.joinState.entityId)
  assert.deepEqual(second.joinState.position, first.joinState.position)
})

async function runJoinProbe (name) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: name
      },
      timeout: 30_000,
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
