# IC side

- Only 10% of hardware capacity is currently being used
- Hardware capability increase over time, cost for executing same number of instructions decrease over time which is equivalent to cycles
- Optimizations and savings from improvements in the base protocol

# Canister side

- Migration to stable memory - accounts for 50% of our current costs
  - Delay this for as long as possible
  - Already burned once, wasted 2-3 months on moving from stable to heap
  - ic-stable-structures - still has breaking bugs moving between minor versions (e.g. v0.3 - v0.4)
  - Schema changes are very easy for heap, but complicated for stable
  - Have to declare size of entries up front which won't work in our case. Dynamic allocator absent
- Recycle dead/inactive canisters
