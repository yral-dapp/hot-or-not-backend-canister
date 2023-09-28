# Current Architecture

```mermaid
  flowchart
    UserClient[User Client Device]
    UserIndex[(User Index)]
    IndividualUserCanister[(Individual User <br>Canister)]
    ContentIndex[(Content Index)]
    GlobalConfiguration[(Global Configuration)]
    DataBackup[(Data Backup)]


    UserIndex -- 1 --> IndividualUserCanister
    IndividualUserCanister -- n --> UserIndex
    IndividualUserCanister -- sync new <br> posts --> ContentIndex
    IndividualUserCanister -- backup before <br> upgrades --> DataBackup
    UserClient -- get provisioned <br> individual canister --> UserIndex
    UserClient -- get content --> ContentIndex
    UserClient -- talk to own <br> and others' canisters --> IndividualUserCanister
```

## Upcoming Architecture

```mermaid
  flowchart
    PlatformOrchestrator[Platform Orchestrator]
    CanisterRegistry[(Canister Registry)]
    UserClient[User Client Device]
    UserIndex[(User Index)]
    IndividualUserCanister[(Individual User <br>Canister)]
    ContentIndex[(Content Index)]

    subgraph OrchestratorSubnet[Orchestrator Subnet]
      PlatformOrchestrator <-- 1 --> CanisterRegistry
    end
    subgraph UserSubnet[User Subnet]
      PlatformOrchestrator --> UserIndex
      UserIndex -- 1 --> IndividualUserCanister
      IndividualUserCanister -- n --> UserIndex
      IndividualUserCanister <-- sync new <br> posts --> ContentIndex
      UserClient -- get provisioned <br> individual canister --> UserIndex
      UserClient -- get content --> IndividualUserCanister
      UserClient -- talk to own <br> and others' canisters --> IndividualUserCanister
    end
```

# Proposed Changes

- Have a configuration canister that pushes changes downstream on every subnet
- Test serializing entire canister contents and then sending it elsewhere to be reinitialized on a different subnet or archived off chain
- Embrace stable memory to eliminate the cost of upgrades
- Embrace subnet ownership to eliminate the cost of new canister creation
- Canister recycling
  - Track user sessions
  - Reclaim if beyond threshold
  - Figure out what to do with the data.

## Scaling

- Subnet splitting
- Manual sharding
  - On chain
  - Off chain
