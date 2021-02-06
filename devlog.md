20200205

current bug: player floats for one frame as staircase transitions down
this causes player to fall through


## TODO:
---

- rewrite propagation engine

- non-square bounds. Set of bounds?
  - this enables more robust horizontal pushing of escalators

- more robust collision detection
  - y colision detection / pushing (+ propagation)
  - perhaps implies some sort of prioritization?
    - whoever was "already there"?

- level editing
- "Box" type (refactor)
- Orientation
- direction
- divide up propagation systems
  - compute graphs, store in a resource?

## BUGS:
---
- prefer existing atop? current behavior is jittery
- weird distinction between no intrinsic velocity vs zero intrinsic velocity?
  - step pushes crate off, doesn't push player off
  - this is just x collision system

- steps push through ground
- can carry item through ground

## 20210204
---
complete:
- ground collision in horizontal stacks prototype


## 20210203
---
complete:
- player
  - interaction -> intrinsic velocity
- fixed: player can tank grounded platform; add grounding check to intermediates?
- horizontal stacks

notes:
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


## 20210202
---
- initial implementation of vertical stacking
- subtle stack overflow bug (06a8515ebdc7ee3c9ff0f33e2a5ae3f8ffaa94a1)
- correction of step / escalator atop / below checking