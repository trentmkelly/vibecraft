export const LOGIN_PROFILE_FIXTURE_KINDS = [
  'fresh',
  'returning',
  'duplicate',
  'op',
  'whitelisted',
  'banned'
]

export function declareLoginScenarioMatrixOwner(name, fixtures) {
  return {
    name,
    fixtures: [...new Set(fixtures)].sort()
  }
}

export function lintLoginScenarioMatrixOwners(scenarios) {
  const issues = []
  for (const scenario of scenarios) {
    if (!scenario.name) issues.push({ scenario: '<unnamed>', message: 'scenario must declare name' })
    if (!Array.isArray(scenario.fixtures) || scenario.fixtures.length === 0) {
      issues.push({ scenario: scenario.name ?? '<unnamed>', message: 'scenario must declare at least one login profile fixture' })
      continue
    }
    for (const fixture of scenario.fixtures) {
      if (!LOGIN_PROFILE_FIXTURE_KINDS.includes(fixture)) {
        issues.push({ scenario: scenario.name, message: `unknown login profile fixture ${fixture}` })
      }
    }
  }
  return {
    ok: issues.length === 0,
    issues
  }
}

export function requiredFixtureCoverage(scenarios) {
  const present = new Set(scenarios.flatMap(scenario => scenario.fixtures ?? []))
  return Object.fromEntries(LOGIN_PROFILE_FIXTURE_KINDS.map(kind => [kind, present.has(kind)]))
}
