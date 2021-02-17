## 20210216

Propagation system rewrite. Store component-wise velocity. Don't worry about min / max for now.

maybe split propagate_velocity into helper fns?

- propagate_intrinsic()
- push()
- carry()

What does updating a propagation look like?

for each, do ground checking (?) / clamping (?)

push: take max absolute value of x (?), leave remaining components
carry: take max value of y, leave other components
intrinsic: set it to new value. should be unique anyway

This system mostly works. We have to be careful about steps pushing each other. Maybe this can be solved when the graph is built?

Moving left; might require clamping

Falling escalator also seems to work but not past 1 frame?

## 20210208
---
implemented custom logic for escalator propagation. It is ugly, but it works well enough for now, maybe?


20200207

test_down fixes weird falling edge case! seems reasonable

todo: fix double x transfer of momentum using `already_visited`

maybe some args of propagation can be bundled somehow?

unit test snippet: https://discord.com/channels/691052431525675048/742884593551802431/808047868425797693

escalator kind of works. steps have weird collisions with other things. escalator falling is broken.

player can't move if atop escalator. I think this is b/c player iv is set to 0, not None.

Add some unit tests and fix some stuff :)

Ok so: falling staircases, what's up with that.

20200205

current bug: player floats for one frame as staircase transitions down
this causes player to fall through

intrinsicVelocity and PropagationResult can become the same thing

Ground can be atop and propagation occurs; this is bad

Can't move if player is atop crate ??? 
More subtle than that. Still not exactly what is up.

Current issue: player can hang from bottom of ground.


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