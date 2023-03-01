# Possible User States

- Unauthenticated
- Authenticated
  - Not bet
    - Enough balance
    - Not enough balance
  - Bet on this slot
    - awaiting results
  - Bet on earlier slot
    - won
    - lost
    - draw

# Possible Post States

- Betting open
  - Slot available out of 100
  - Current Slot full
- All betting periods transpired

# Visualization

```mermaid
flowchart TD
    UnauthenticatedUser[Unauthenticated user] -- Login --> AuthenticatedUserNotBet[Authenticated user who hasn't bet on this post yet]
    UnauthenticatedUser -- Login --> AuthenticatedUserBetOnCurrentSlot[Authenticated user who has bet on this slot]
    UnauthenticatedUser -- Login --> AuthenticatedUserBetOnEarlierSlot[Authenticated user who has bet on an earlier slot]
    AuthenticatedUserNotBet -- Current Slot full --> PostCurrentSlotFull[Eligible authenticated user can't bet, current slot full]
    AuthenticatedUserNotBet -- Slot available --> PostCurrentSlotAvailable[Current slot in post is available]
    PostCurrentSlotAvailable -- Not enough balance --> AuthenticatedUserNotEnoughBalance[Authenticated user does not have enough balance]
    PostCurrentSlotAvailable -- Enough balance --> AuthenticatedUserEnoughBalance[Authenticated user has enough balance]
    AuthenticatedUserEnoughBalance -- Place bet --> AuthenticatedUserBetOnCurrentSlot
    AuthenticatedUserBetOnCurrentSlot -- Slot contest over --> AuthenticatedUserBetOnEarlierSlot[Authenticated user who has bet on an earlier slot with results published]
```
