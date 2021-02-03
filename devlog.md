## TODO:
---
- more robust collision detection
  - y colision detection / pushing (+ propagation)
- horizontal stacks
- level editing

## BUGS:
---
- player can tank grounded platform; add grounding check to intermediates?


## 20210203
---
- player
  - interaction -> intrinsic velocity

## 20210202
---
- initial implementation of vertical stacking
- subtle stack overflow bug (06a8515ebdc7ee3c9ff0f33e2a5ae3f8ffaa94a1)
- correction of step / escalator atop / below checking