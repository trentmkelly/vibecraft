import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 parallel offline logins isolate profile, configuration, play, and keepalive state', { timeout: 40_000 }, async () => {
  const usernames = ['ParallelA', 'ParallelB', 'ParallelC', 'ParallelD']
  const started = Promise.all(usernames.map(username => runJoinProbe(username)))
  const results = await started

  assert.equal(results.length, usernames.length)
  assert.deepEqual(results.map(result => result.ok), usernames.map(() => true))

  const uuids = new Set()
  for (const [index, result] of results.entries()) {
    const username = usernames[index]
    assert.equal(result.joinState.profile.name, username)
    assert.equal(result.joinState.profile.uuid, offlineUuid(username))
    assert.equal(result.joinState.entityId, 1)
    assert.equal(result.joinState.dimension, 'minecraft:overworld')
    assert.equal(result.joinState.lastReceivedChunk, 8)
    assert.ok(result.config.some(packet => packet.id === 3), `${username} should finish configuration`)
    assert.ok(result.play.some(packet => packet.id === 49), `${username} should reach play login`)
    assert.ok(result.play.some(packet => packet.id === 70), `${username} should receive its own tab-list profile`)
    uuids.add(result.joinState.profile.uuid)
  }

  assert.equal(uuids.size, usernames.length)
})

async function runJoinProbe (username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: username
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
