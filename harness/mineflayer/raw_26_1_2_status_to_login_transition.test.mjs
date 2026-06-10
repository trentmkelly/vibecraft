import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 status socket cleanup does not corrupt immediate offline login', async () => {
  const status = await runJsonProbe('raw_26_1_2_status_probe.mjs')
  assert.equal(status.ok, true)
  assert.equal(status.status.version.protocol, 775)

  const login = await runJsonProbe('raw_26_1_2_join_probe.mjs', {
    VIBECRAFT_USERNAME: 'StatusLogin'
  })
  assert.equal(login.ok, true)
  assert.ok(login.config.some(packet => packet.id === 3), 'expected finish configuration packet')
  assert.ok(login.play.some(packet => packet.id === 49), 'expected play login packet')
})

async function runJsonProbe (script, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    [script],
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
