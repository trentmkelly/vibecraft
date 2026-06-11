import test from 'node:test'
import assert from 'node:assert/strict'
import {
  artifactDiffSummary,
  diffOfflineLoginArtifacts
} from './offline_artifact_diff.mjs'

test('diffOfflineLoginArtifacts ignores volatile ports, timestamps, temp paths, and seeds', () => {
  const diff = diffOfflineLoginArtifacts(
    loginRun('/tmp/official-artifacts', 25565, 111, '2026-05-17T12:00:00Z'),
    loginRun('/tmp/vibecraft-artifacts', 25566, 222, '2026-05-17T12:00:01Z')
  )
  assert.deepEqual(diff, [])
  assert.deepEqual(artifactDiffSummary(diff), { ok: true, paths: [] })
})

test('diffOfflineLoginArtifacts reports real vanilla-vs-VibeCraft artifact differences', () => {
  const official = loginRun('/tmp/official-artifacts', 25565, 111, '2026-05-17T12:00:00Z')
  const vibecraft = loginRun('/tmp/vibecraft-artifacts', 25566, 222, '2026-05-17T12:00:01Z')
  vibecraft.artifacts.events = [{ name: 'kicked', summary: ['unexpected'] }]

  const diff = diffOfflineLoginArtifacts(official, vibecraft)
  assert.deepEqual(artifactDiffSummary(diff), { ok: false, paths: ['events'] })
})

function loginRun(root, port, seed, timestamp) {
  return {
    root,
    port,
    seed,
    artifacts: {
      root,
      events: [{ name: 'spawn', summary: ['ok'] }],
      logs: [{ stream: 'stdout', text: `${timestamp} started ${root} on ${port} seed ${seed}\n` }],
      serverProperties: `server-port=${port}\nlevel-seed=${seed}\nonline-mode=false\n`,
      eula: 'eula=true\n'
    }
  }
}
