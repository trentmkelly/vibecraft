import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 login cancellation leaves same username able to reconnect', async () => {
  for (const phase of ['login_success', 'registry_sync', 'first_chunk']) {
    const username = `Cancel${phase.replaceAll('_', '').slice(0, 9)}`
    const cancelled = await runJoinProbe({
      RUSTCRAFT_USERNAME: username,
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: phase
    })
    assert.equal(cancelled.ok, true)
    assert.equal(cancelled.aborted, true)
    assert.equal(cancelled.phase, phase)

    const retry = await runJoinProbe({ RUSTCRAFT_USERNAME: username })
    assert.equal(retry.ok, true)
    assert.ok(retry.config.some(packet => packet.id === 3), `expected ${phase} retry to finish configuration`)
    assert.ok(retry.play.some(packet => packet.id === 49), `expected ${phase} retry to reach play`)
  }
})

async function runJoinProbe (env) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        ...env
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
