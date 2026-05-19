import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  CORE_WORKFLOWS,
  evaluateReleaseGate,
  formatReleaseGateReport
} from './release_gate.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

test('CORE_WORKFLOWS requires core vanilla parity release surfaces', () => {
  assert.deepEqual(CORE_WORKFLOWS, [
    'offline-login',
    'packet-flow',
    'offline-regression',
    'vanilla-compatibility',
    'gameplay-smoke',
    'known-parity-regressions'
  ])
})

test('evaluateReleaseGate passes only when all core workflows pass and no known parity regressions remain', () => {
  const gate = evaluateReleaseGate({
    loginGate: { ok: true },
    packetFlow: { ok: true, checks: [{ name: 'offlineLogin', ok: true }] },
    offlineRegression: { ok: true, regression: { checks: [{ name: 'playStateStall', ok: true }] } },
    vanillaCompatibility: { ok: true, workflows: [{ name: 'client-join', ok: true }] },
    gameplaySmoke: { ok: true, steps: { movement: true } },
    knownRegressions: []
  })

  assert.equal(gate.ok, true)
  assert.equal(gate.checks.length, CORE_WORKFLOWS.length)
})

test('evaluateReleaseGate fails when a workflow fails or known parity regressions remain', () => {
  const gate = evaluateReleaseGate({
    loginGate: { ok: true },
    packetFlow: { ok: false, checks: [{ name: 'keepAlive', ok: false }] },
    offlineRegression: { ok: true, regression: { checks: [] } },
    vanillaCompatibility: { ok: true, workflows: [] },
    gameplaySmoke: { ok: true, steps: {} },
    knownRegressions: ['registry order mismatch']
  })

  assert.equal(gate.ok, false)
  assert.equal(gate.checks.find(check => check.name === 'packet-flow').ok, false)
  assert.equal(gate.checks.find(check => check.name === 'known-parity-regressions').ok, false)
})

test('formatReleaseGateReport emits stable pass and fail lines', () => {
  const gate = evaluateReleaseGate({
    loginGate: { ok: true },
    packetFlow: { ok: true },
    offlineRegression: { ok: true },
    vanillaCompatibility: { ok: true },
    gameplaySmoke: { ok: true },
    knownRegressions: []
  })

  assert.match(formatReleaseGateReport(gate), /PASS offline-login/)
  assert.match(formatReleaseGateReport(gate), /PASS known-parity-regressions/)
})

test('release compatibility notes document incomplete and intentional differences', async () => {
  const notes = await readFile(path.join(repoRoot, 'docs', 'COMPATIBILITY.md'), 'utf8')

  assert.match(notes, /Known incomplete areas:/)
  assert.match(notes, /Intentional deviations:/)
  assert.match(notes, /Play-state gameplay is not implemented\./)
  assert.match(notes, /Online-mode authentication and secure chat are not implemented\./)
  assert.match(notes, /DataFixer-compatible upgrades are not implemented\./)
  assert.match(notes, /The dedicated server Swing GUI is intentionally unsupported/)
  assert.match(notes, /Before any `CHECKLIST\.md` item is checked/)
})
