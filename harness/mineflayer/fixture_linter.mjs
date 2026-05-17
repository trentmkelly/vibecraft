export function lintMineflayerFixture(fixture) {
  const issues = []
  requireString(issues, fixture.version, 'version', 'scenario must pin an explicit Mineflayer/Minecraft version')
  requireObject(issues, fixture.serverProperties, 'serverProperties', 'scenario must declare server.properties overrides')
  if (fixture.serverProperties && fixture.serverProperties['online-mode'] !== 'false') {
    issues.push(issue('serverProperties.online-mode', 'offline-mode scenarios must set online-mode=false'))
  }
  lintProfiles(issues, fixture.profiles)
  if (!Number.isInteger(fixture.timeoutMs) || fixture.timeoutMs <= 0) {
    issues.push(issue('timeoutMs', 'scenario must declare a positive timeout budget in milliseconds'))
  }
  lintPacketCapture(issues, fixture.packetCapture)
  lintVanillaComparison(issues, fixture.vanillaComparison)
  return {
    ok: issues.length === 0,
    issues
  }
}

export function assertMineflayerFixture(fixture) {
  const result = lintMineflayerFixture(fixture)
  if (!result.ok) {
    throw new Error(formatFixtureLint(result))
  }
  return fixture
}

export function formatFixtureLint(result) {
  if (result.ok) return 'fixture lint passed'
  return result.issues.map(item => `${item.path}: ${item.message}`).join('\n')
}

function lintProfiles(issues, profiles) {
  if (!Array.isArray(profiles) || profiles.length === 0) {
    issues.push(issue('profiles', 'scenario must include at least one offline profile'))
    return
  }
  profiles.forEach((profile, index) => {
    requireString(issues, profile.username, `profiles.${index}.username`, 'profile must include username')
    requireString(issues, profile.expectedUuid, `profiles.${index}.expectedUuid`, 'profile must include expected vanilla offline UUID')
  })
}

function lintPacketCapture(issues, packetCapture) {
  if (!packetCapture || typeof packetCapture !== 'object') {
    issues.push(issue('packetCapture', 'scenario must declare packet capture policy'))
    return
  }
  if (packetCapture.enabled !== true) {
    issues.push(issue('packetCapture.enabled', 'packet capture must be explicitly enabled'))
  }
  const states = packetCapture.states ?? []
  for (const required of ['login', 'configuration', 'play']) {
    if (!states.includes(required)) {
      issues.push(issue('packetCapture.states', `packet capture must include ${required} state`))
    }
  }
}

function lintVanillaComparison(issues, vanillaComparison) {
  const allowed = new Set(['required', 'optional', 'disabled'])
  if (!vanillaComparison || typeof vanillaComparison !== 'object') {
    issues.push(issue('vanillaComparison', 'scenario must declare vanilla comparison mode'))
    return
  }
  if (!allowed.has(vanillaComparison.mode)) {
    issues.push(issue('vanillaComparison.mode', 'vanilla comparison mode must be required, optional, or disabled'))
  }
}

function requireString(issues, value, path, message) {
  if (typeof value !== 'string' || value.length === 0) issues.push(issue(path, message))
}

function requireObject(issues, value, path, message) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) issues.push(issue(path, message))
}

function issue(path, message) {
  return { path, message }
}
