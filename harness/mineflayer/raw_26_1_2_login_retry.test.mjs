import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 offline login retry recovers immediately after a forced first-attempt disconnect', { timeout: 90_000 }, async () => {
  const phases = ['login_success', 'registry_sync', 'first_chunk']

  for (const phase of phases) {
    const username = `Rt${phase.replaceAll('_', '').slice(0, 8)}${crypto.randomUUID().replaceAll('-', '').slice(0, 5)}`
    const failed = await runJoinProbe(username, { RUSTCRAFT_RAW_PROBE_ABORT_AFTER: phase })
    assert.equal(failed.ok, true)
    assert.equal(failed.aborted, true)
    assert.equal(failed.phase, phase)

    const retry = await runJoinProbe(username)
    assert.equal(retry.ok, true)
    assert.equal(retry.joinState.profile.name, username)
    assert.ok(retry.configPacketCount > 0, `${phase} retry should receive config packets`)
    assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
    assert.ok(retry.joinState.initialChunkCount > 0, `${phase} retry should receive initial chunks`)
    assert.equal(retry.joinState.lastReceivedChunk, retry.joinState.initialChunkCount - 1, `${phase} retry should receive complete initial chunk batch`)
  }
})

async function runJoinProbe (username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: username,
        RUSTCRAFT_RAW_PROBE_OUTPUT: 'summary',
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
