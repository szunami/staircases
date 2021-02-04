The idea of intrinsic velocity is to enable the movement that each component contributes to bubble up / be distinguished. The current velocity propagation mechanism is brittle and overly complex.

To wit: intrinsic velocity should be applied directly to self?

initialize velocity system: 

if intrinsic exists, use it
if not, ???
0 is a bad default value, so is inf, so is gravity
Velocity as Option?


x propagation only propagates -> doesn't have to worry about steps

y propagation only propagates -> doesn't have to care about sequence of sums

OK this is slightly more complicated b/c we don't want to revisit nodes > 1 time, but it mostly works !


## TODO:
---
- more robust collision detection
  - y colision detection / pushing (+ propagation)
  - perhaps implies some sort of prioritization?
    - whoever was "already there"?
- horizontal stacks
- level editing

## BUGS:
---
- prefer existing atop? current behavior is jittery
- issue: player double counts horizontal velocity. fix this


## 20210203
---
- player
  - interaction -> intrinsic velocity
- fixed: player can tank grounded platform; add grounding check to intermediates?


## 20210202
---
- initial implementation of vertical stacking
- subtle stack overflow bug (06a8515ebdc7ee3c9ff0f33e2a5ae3f8ffaa94a1)
- correction of step / escalator atop / below checking