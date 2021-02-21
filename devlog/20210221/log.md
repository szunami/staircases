## TODO:

Investigate clipping!
Hypotheses:
- corner case

[src/main.rs:43] bottom = -100.0
[src/main.rs:43] left = -300.0
[src/main.rs:43] v.0 = Some(
    Vec2(
        0.0,
        -1.0,
    ),
)

OK so actually this is related to the double carry. The downward step has no left x, so test_left wasn't called.
Solution: remove that conditional logic.

## Done: