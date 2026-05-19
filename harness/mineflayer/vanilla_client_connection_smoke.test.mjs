import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createVanillaClientConnectionSmokePlan,
  xdotoolConnectionSteps
} from './vanilla_client_connection_smoke.mjs'

test('createVanillaClientConnectionSmokePlan describes the vanilla Xephyr join workflow and artifacts', () => {
  const plan = createVanillaClientConnectionSmokePlan({
    display: ':8',
    gameDir: '/tmp/client-game',
    artifactsDir: '/tmp/client-artifacts',
    host: 'localhost',
    port: 25566
  })

  assert.equal(plan.name, 'vanilla-client-xephyr-connection-smoke')
  assert.equal(plan.client.display, ':8')
  assert.equal(plan.client.gameDir, '/tmp/client-game')
  assert.equal(plan.server, 'localhost:25566')
  assert.equal(plan.joinedScreenshotPath, '/tmp/client-artifacts/joined-local-server.png')
  assert.equal(plan.collectedLatestLogPath, '/tmp/client-artifacts/latest.log')
  assert.equal(plan.collectedCrashReportsDir, '/tmp/client-artifacts/crash-reports')
  assert.ok(plan.phases.includes('open Multiplayer from main menu'))
  assert.ok(plan.phases.includes('wait for terrain to render'))
  assert.ok(plan.phases.includes('collect latest.log and crash reports'))
})

test('xdotoolConnectionSteps focuses Minecraft, opens multiplayer, selects server, and joins', () => {
  const plan = createVanillaClientConnectionSmokePlan({
    artifactsDir: '/tmp/client-artifacts'
  })
  const steps = xdotoolConnectionSteps(plan, {
    menuDelayMs: 100,
    multiplayerButton: { x: 10, y: 20 },
    serverRow: { x: 30, y: 40 },
    joinButton: { x: 50, y: 60 }
  })

  assert.deepEqual(steps.map(step => step.description), [
    'focus Minecraft window',
    'wait for focus',
    'open Multiplayer',
    'wait for Multiplayer list',
    'select saved local server',
    'wait for server row selection',
    'join selected server'
  ])
  assert.deepEqual(steps[0].args, ['search', '--onlyvisible', '--name', 'Minecraft', 'windowactivate'])
  assert.deepEqual(steps[2].args, ['mousemove', '10', '20', 'click', '1'])
  assert.deepEqual(steps[4].args, ['mousemove', '30', '40', 'click', '1'])
  assert.deepEqual(steps[6].args, ['mousemove', '50', '60', 'click', '1'])
})
