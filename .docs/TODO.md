# TODO

- [X] Change backend from InMemoryStore to database access
- [X] Use JWT on the frontend and not use local storage
- [X] Re-organize backend into MVC like structure
- [X] Deprecated InMemoryRepository for backend
- [ ] Improve visualization of database data (probably add dbeaver or some gui in devenv.nix)
- [ ] Allows tampermonkey to show app data inside leetcode and neetcode
- [ ] Create calendar view or somekind of it
- [ ] Make the SRS delays not based on only the problem, but also account for pattern + complexity of the problem
- [ ] Ship backend JSON logs to a local searchable stack (Elastic/OpenSearch/Loki + UI) and index by requestId
- [ ] Add OpenTelemetry (traces + logs) with collector/exporters and correlate logs/traces by requestId/trace context
- [ ] Connect tampermonkey via api key with the frontend solution

# Bugs

- [ ] Neetcode problem name in dashboard shows just "leetcode"