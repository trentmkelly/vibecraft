import test from 'node:test'
import assert from 'node:assert/strict'
import {
  commandScenarioManifest,
  commandPermissionReloadManifest,
  commandPermissionReloadScenario,
  loginGatedCommandManifest,
  loginGatedCommandScenarios,
  listLoginStateManifest,
  listLoginStateScenarios,
  observeCommandFeedback,
  offlineCommandScenarios,
  operatorCommandSmokeManifest,
  operatorCommandSmokeScenarios,
  runCommandScenario,
  summarizeCommandPermissionReloadEvidence,
  summarizeLoginGatedCommandEvidence,
  summarizeListLoginStateEvidence,
  summarizeTeleportCommandEvidence,
  teleportCommandExecutionScenario
} from './command_scenarios.mjs'

test('offlineCommandScenarios cover required Mineflayer command surface', () => {
  assert.deepEqual(offlineCommandScenarios().map(scenario => scenario.command), [
    '/list',
    '/tell RustCraftBot1 hello',
    '/msg RustCraftBot1 hello',
    '/me waves',
    '/help list',
    '/seed',
    '/gamemode creative RustCraftBot',
    '/gamemode creative'
  ])
})

test('offlineCommandScenarios include feedback expectations and permission-denied case', () => {
  const scenarios = offlineCommandScenarios({ primary: 'Steve', secondary: 'Alex' })
  assert.equal(scenarios.find(entry => entry.command === '/seed').minPermission, 2)
  assert.equal(scenarios.find(entry => entry.command === '/gamemode creative').expectDenied, true)
  assert.equal(
    scenarios.find(entry => entry.command === '/gamemode creative Steve').expectedFeedbackKey,
    'commands.gamemode.success.other'
  )
})

test('observeCommandFeedback matches translatable feedback keys and permission denial', () => {
  assert.deepEqual(observeCommandFeedback({
    timeline: [{ name: 'message', summary: ['commands.list.players'] }]
  }, {
    expectedFeedbackKey: 'commands.list.players'
  }), {
    matched: true,
    denied: false,
    messages: [['commands.list.players']]
  })
  assert.equal(observeCommandFeedback({
    timeline: [{ name: 'message', summary: ['commands.generic.permission'] }]
  }, {
    expectedFeedbackKey: 'commands.seed.success'
  }).denied, true)
})

test('runCommandScenario dispatches commands and reports observed feedback', async () => {
  const sent = []
  const session = {
    timeline: [{ name: 'message', summary: ['commands.help.success'] }]
  }
  const result = await runCommandScenario(session, {
    command: '/help list',
    expectedFeedbackKey: 'commands.help.success',
    expectDenied: false
  }, {
    dispatch: (_session, command) => {
      sent.push(command)
      return { command }
    }
  })
  assert.deepEqual(sent, ['/help list'])
  assert.equal(result.ok, true)
})

test('runCommandScenario treats permission denied feedback as expected only for denied scenarios', async () => {
  const session = {
    timeline: [{ name: 'message', summary: ['commands.generic.permission'] }]
  }
  assert.equal((await runCommandScenario(session, {
    command: '/gamemode creative',
    expectedFeedbackKey: 'commands.generic.permission',
    expectDenied: true
  }, {
    dispatch: () => ({})
  })).ok, true)
})

test('commandScenarioManifest serializes offline command test metadata', () => {
  const manifest = commandScenarioManifest({ primary: 'Steve', secondary: 'Alex' })
  assert.equal(manifest.mode, 'offline')
  assert.equal(manifest.auth, 'offline')
  assert.equal(manifest.commands.length, 8)
  assert.ok(manifest.commands.some(command => command.command === '/tell Alex hello'))
})

test('operatorCommandSmokeScenarios cover required op command smoke surface', () => {
  const commands = operatorCommandSmokeScenarios({ primary: 'Steve', secondary: 'Alex' })
    .map(entry => entry.command.split(' ')[0])

  assert.deepEqual(commands, [
    '/op',
    '/deop',
    '/whitelist',
    '/ban',
    '/pardon',
    '/gamemode',
    '/tp',
    '/give',
    '/effect'
  ])
})

test('operatorCommandSmokeManifest records feedback, permission gates, and reconnect-visible state', () => {
  const manifest = operatorCommandSmokeManifest({ primary: 'Steve', secondary: 'Alex' })
  assert.equal(manifest.source, 'op-bot')
  assert.equal(manifest.commands.length, 9)
  assert.ok(manifest.commands.every(command => command.minPermission >= 2))
  assert.ok(manifest.commands.some(command => command.expectedFeedbackKey === 'commands.op.success'))
  assert.ok(manifest.commands.some(command => command.expectedFeedbackKey === 'commands.effect.give.success.single'))
  assert.ok(manifest.commands.filter(command => command.reconnectVisibleState).length >= 5)
})

test('listLoginStateManifest covers console and bot list counts across login lifecycle', () => {
  const manifest = listLoginStateManifest({ primary: 'Steve' })
  assert.equal(manifest.command, '/list')
  assert.deepEqual(manifest.scenarios.map(scenario => scenario.name), [
    'console-during-login',
    'bot-during-login',
    'after-join',
    'after-duplicate-replacement',
    'after-disconnect'
  ])
  assert.deepEqual(manifest.scenarios.map(scenario => scenario.expectedPlayerCount), [0, 0, 1, 1, 0])
})

test('summarizeListLoginStateEvidence validates player counts and names for every list phase', () => {
  const scenarios = listLoginStateScenarios({ primary: 'Steve' })
  const evidence = {
    timeline: [
      { name: 'list_command', summary: ['console-during-login', { count: 0, names: [] }] },
      { name: 'list_command', summary: ['bot-during-login', { count: 0, names: [] }] },
      { name: 'list_command', summary: ['after-join', { count: 1, names: ['Steve'] }] },
      { name: 'list_command', summary: ['after-duplicate-replacement', { count: 1, names: ['Steve'] }] },
      { name: 'list_command', summary: ['after-disconnect', { count: 0, names: [] }] }
    ]
  }

  assert.equal(summarizeListLoginStateEvidence(evidence, scenarios).ok, true)
  evidence.timeline[2].summary[1].names = ['Alex']
  assert.equal(summarizeListLoginStateEvidence(evidence, scenarios).ok, false)
})

test('teleportCommandExecutionScenario records command, position, feedback, and denial requirements', () => {
  const scenario = teleportCommandExecutionScenario({
    username: 'Steve',
    position: { x: 4, y: 70, z: 4 }
  })

  assert.equal(scenario.command, '/tp Steve 4 70 4')
  assert.deepEqual(scenario.requiredSteps, [
    'teleport-command-issued',
    'position-correction-observed',
    'success-feedback',
    'non-op-permission-failure'
  ])
})

test('summarizeTeleportCommandEvidence validates observed position, success feedback, and permission failure', () => {
  const scenario = teleportCommandExecutionScenario({
    username: 'Steve',
    position: { x: 4, y: 70, z: 4 }
  })
  const evidence = {
    timeline: [
      { name: 'teleport_command', summary: ['issued', { command: scenario.command }] },
      { name: 'position', summary: ['Steve', { x: 4, y: 70, z: 4 }] },
      { name: 'message', summary: ['commands.teleport.success.location.single'] },
      { name: 'message', summary: ['commands.generic.permission'] }
    ]
  }

  assert.equal(summarizeTeleportCommandEvidence(evidence, scenario).ok, true)
  evidence.timeline[1].summary[1].z = 5
  assert.equal(summarizeTeleportCommandEvidence(evidence, scenario).ok, false)
})

test('loginGatedCommandManifest covers commands that must wait for play readiness', () => {
  const manifest = loginGatedCommandManifest({ primary: 'Steve', secondary: 'Alex' })

  assert.equal(manifest.readinessBoundary, 'play-state-ready')
  assert.deepEqual(manifest.commands.map(command => command.command), [
    '/list',
    '/tell Alex ready',
    '/gamemode creative Steve',
    '/tp Steve 0 80 0'
  ])
  assert.ok(manifest.commands.every(command => command.preReadyBlocked))
})

test('summarizeLoginGatedCommandEvidence requires blocked pre-ready and successful post-ready feedback', () => {
  const scenarios = loginGatedCommandScenarios({ primary: 'Steve', secondary: 'Alex' })
  const evidence = {
    timeline: scenarios.map(scenario => ({
      name: 'login_gated_command',
      summary: [scenario.command, {
        preReadyBlocked: true,
        postReadyRan: true,
        feedback: scenario.expectedFeedbackKey
      }]
    }))
  }

  assert.equal(summarizeLoginGatedCommandEvidence(evidence, scenarios).ok, true)
  evidence.timeline[0].summary[1].preReadyBlocked = false
  assert.equal(summarizeLoginGatedCommandEvidence(evidence, scenarios).ok, false)
})

test('commandPermissionReloadManifest covers ops file edits and op/deop command tree deltas', () => {
  const manifest = commandPermissionReloadManifest({ username: 'Steve' })

  assert.deepEqual(manifest.files, ['ops.json'])
  assert.deepEqual(manifest.commands, ['/op', '/deop'])
  assert.deepEqual(manifest.steps, [
    'ops-json-edited',
    'op-command-tree-delta',
    'deop-command-tree-delta',
    'reconnect-permission-state',
    'denied-feedback'
  ])
})

test('summarizeCommandPermissionReloadEvidence requires reload persistence and denied feedback', () => {
  const scenario = commandPermissionReloadScenario({ username: 'Steve' })
  const evidence = {
    timeline: [
      { name: 'command_permission_reload', summary: ['ops-json-edited', { written: true }] },
      { name: 'command_permission_reload', summary: ['op-command-tree-delta', { visible: true }] },
      { name: 'command_permission_reload', summary: ['deop-command-tree-delta', { visible: false }] },
      { name: 'command_permission_reload', summary: ['reconnect-permission-state', { persisted: true }] },
      { name: 'message', summary: ['commands.generic.permission'] }
    ]
  }

  assert.equal(summarizeCommandPermissionReloadEvidence(evidence, scenario).ok, true)
  evidence.timeline[3].summary[1].persisted = false
  assert.equal(summarizeCommandPermissionReloadEvidence(evidence, scenario).ok, false)
})
