# Scenarios

- Anonymous/authenticated user queries a post for hot or not details
- Authenticated user places a bet on a post
- Betting slot ends and outcomes are calculated
  - Outcomes are stored in the creator's and bet maker's canister
  - Creator and bet maker wallets are updated accordingly

# Visualization

```mermaid
---
title: Actor flow for betting on a Hot or Not Post
---
flowchart
  UserClient[User]
  PostCreatorCanister[Post creator canister]
  UserOwnCanister[User own canister]

  UserClient -- 1. Query post detail --> PostCreatorCanister
  PostCreatorCanister -- 2. Respond with post details --> UserClient
  UserClient -- 3. Try to bet --> UserOwnCanister
  UserOwnCanister -- 4. Check eligibility <br>and forward details --> PostCreatorCanister
  PostCreatorCanister -- 5. Add bet details<br> and respond --> UserOwnCanister
  UserOwnCanister -- 6. Update internally <br> and send acknowledgement --> UserClient
  PostCreatorCanister -- 7. Tabulate results --> PostCreatorCanister
  PostCreatorCanister -- 8. Credit creator's <br>wallet --> PostCreatorCanister
  PostCreatorCanister -- 9. Update user's canister <br> with outcome --> UserOwnCanister
  UserOwnCanister -- 10. Credit/debit <br>user wallet --> UserOwnCanister
  UserClient -- 11. Query outcome --> UserOwnCanister
  UserClient -- 12. Query wallet --> UserOwnCanister
```

## Query post details

```mermaid
flowchart
  UserClient[User Client]
  UserLoggedIn{Is user logged in?}
  HasUserBetOnCurrentPost[Additional Detail: <br>Has user bet on current post?]
  BetDetails[Bet Details]
  BettingOpen[Betting Open]
  CurrentBetDetails[/Current details: <br>Betting start time <br>Current Slot <br>Current Room <br>Current participants/]
  BettingClosed[Betting Closed]

  UserClient --> UserLoggedIn
  UserLoggedIn -- No --> BetDetails
  UserLoggedIn -- Yes --> HasUserBetOnCurrentPost --> BetDetails
  BetDetails -- Within 48 hours --> BettingOpen
  BettingOpen --> CurrentBetDetails
  CurrentBetDetails --> UserClient
  BetDetails -- Post 48 hours --> BettingClosed
  BettingClosed --> UserClient
```

## Place a bet

```mermaid
flowchart
  AuthenticatedUser[Authenticated User]
  BetDetails[/Bet Details: <br>- Amount <br>- Hot or not/]
  Error::InsufficientBalance[Error <br>Insufficient Balance]
  HasEnoughTokensToBet{Does user have <br>enough tokens to bet?}
  PostCreatorCanister[Post creator canister]
  StakeTokens[/Stake tokens/]
  UserOwnCanister[User own canister]

  AuthenticatedUser -- 1. Place bet --> UserOwnCanister
  UserOwnCanister --> HasEnoughTokensToBet
  HasEnoughTokensToBet -- No --> Error::InsufficientBalance
  HasEnoughTokensToBet -- Yes --> StakeTokens
  StakeTokens -- 2. Tokens deducted <br>from balance --> BetDetails
  BetDetails -- 3. Place bet --> PostCreatorCanister
  PostCreatorCanister -- - Add tokens to pot <br>- Record bet details<br>- Respond with <br>successful status --> UserOwnCanister
  UserOwnCanister -- 4. Update internal <br>state --> UserOwnCanister
  UserOwnCanister -- 5. Send acknowledgement --> AuthenticatedUser
```

## Tabulate outcomes and update wallets

```mermaid
flowchart
  PostCreatorCanister[Post creator canister]
  InputTabulation[/48 slots <br>Tabulate all room outcomes <br>in respective slots/]
  UserOwnCanister[User own canister]
  UserClient[User]
  PostCreatorClient[Post creator]

  PostCreatorCanister -- 1. Every hour <br>perform tabulations --> InputTabulation
  InputTabulation -- 2. Save Outcomes --> PostCreatorCanister
  PostCreatorCanister -- 3. Update creator's wallet <br>with commisions --> PostCreatorCanister
  PostCreatorCanister -- 4. Update participant canisters <br>with outcome --> UserOwnCanister
  PostCreatorCanister -- 5. Update participant wallets <br>with outcome --> UserOwnCanister
  UserClient -- 6. Query outcome --> UserOwnCanister
  UserClient -- 7. Query wallet --> UserOwnCanister
  PostCreatorClient -- 8. Query wallet --> PostCreatorCanister
```
