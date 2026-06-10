import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createVanillaClientXephyrPlan,
  redactMinecraftClientArgv,
  sanitizeMinecraftClientArgv
} from './vanilla_client_xephyr.mjs'

test('createVanillaClientXephyrPlan records nested display, isolated gameDir, and artifact paths', () => {
  const plan = createVanillaClientXephyrPlan({
    display: ':7',
    screen: '1024x768',
    gameDir: '/tmp/vibecraft-client',
    artifactsDir: '/tmp/vibecraft-artifacts'
  })

  assert.equal(plan.name, 'vanilla-client-xephyr-oracle')
  assert.equal(plan.display, ':7')
  assert.equal(plan.screen, '1024x768')
  assert.equal(plan.gameDir, '/tmp/vibecraft-client')
  assert.equal(plan.screenshotPath, '/tmp/vibecraft-artifacts/xephyr-root.png')
  assert.equal(plan.launchLogPath, '/tmp/vibecraft-artifacts/launch.json')
  assert.equal(plan.clientLogPath, '/tmp/vibecraft-client/logs/latest.log')
  assert.equal(plan.environment.DISPLAY, ':7')
  assert.equal(plan.environment.GLFW_PLATFORM, 'x11')
  assert.equal(plan.environment.XDG_SESSION_TYPE, 'x11')
  assert.ok(plan.requirements.includes('no full argv logging because vanilla launch args contain auth material'))
})

test('sanitizeMinecraftClientArgv rewrites gameDir and removes quickPlayPath', () => {
  const argv = [
    '/usr/bin/java',
    '-cp',
    'minecraft.jar',
    'net.minecraft.client.main.Main',
    '--gameDir',
    '/home/trent/.minecraft',
    '--quickPlayPath',
    '/tmp/quickplay.json',
    '--accessToken',
    'secret-token'
  ]

  assert.deepEqual(sanitizeMinecraftClientArgv(argv, { gameDir: '/tmp/isolated' }), [
    '/usr/bin/java',
    '-cp',
    'minecraft.jar',
    'net.minecraft.client.main.Main',
    '--gameDir',
    '/tmp/isolated',
    '--accessToken',
    'secret-token'
  ])
})

test('sanitizeMinecraftClientArgv adds gameDir when launcher argv lacks one', () => {
  const argv = ['/usr/bin/java', 'net.minecraft.client.main.Main']

  assert.deepEqual(sanitizeMinecraftClientArgv(argv, { gameDir: '/tmp/isolated' }), [
    '/usr/bin/java',
    'net.minecraft.client.main.Main',
    '--gameDir',
    '/tmp/isolated'
  ])
})

test('redactMinecraftClientArgv hides auth-bearing values before writing launch artifacts', () => {
  assert.deepEqual(redactMinecraftClientArgv([
    '/usr/bin/java',
    'net.minecraft.client.main.Main',
    '--accessToken',
    'secret-token',
    '--clientId',
    'client-secret',
    '--username',
    'Player'
  ]), [
    '/usr/bin/java',
    'net.minecraft.client.main.Main',
    '--accessToken',
    '<redacted>',
    '--clientId',
    '<redacted>',
    '--username',
    'Player'
  ])
})
