import { offlineUuid } from './runner.mjs'

export function evaluateFixtureUuidSanity(fixture, observations = {}) {
  const results = fixture.profiles.map(profile => {
    const expected = profile.expectedUuid ?? offlineUuid(profile.username)
    return {
      username: profile.username,
      expectedUuid: expected,
      beforeStart: expected === offlineUuid(profile.username),
      vanillaLogin: observations.vanilla?.[profile.username] === expected,
      vibecraftLogin: observations.vibecraft?.[profile.username] === expected
    }
  })
  return {
    ok: results.every(result => result.beforeStart && result.vanillaLogin && result.vibecraftLogin),
    results
  }
}
