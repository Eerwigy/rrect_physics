# pvw_rrect_physics

This is a simple physics implementation with axis-aligned rounded rectangles that I'm using for my game Pancakes vs Waffles. I do not recommend using this in your own projects because it is probably filled with bugs.

## Current limitations

- No tests or documentation
- It becomes laggy when many entities are close together

## Examples

### Simple

This example has a player, two pushable boxes (one heavy and one not heavy) and an immovable wall.

To run:
```bash
cargo r --example simple
```

Or run with hitboxes shown:
```bash
cargo r --feature gizmos --example simple
```

### Stress test

In this example, you can spawn an entity by left-clicking or 10 entities by right-clicking. This is useful for testing performance.

To run:
```bash
cargo r --example stress_test
```

Or run with hitboxes shown:
```bash
cargo r --feature gizmos --example stress_test
```
