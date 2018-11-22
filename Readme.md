# HairSim in Amethyst

## Mass-Spring

### Mass-Spring Model

There should be components that represents MassNode and Spring parameters, together they holds all the data to calculate positions of nodes that represent a hair.

```rust
#[derive(Component, Debug)]
struct Mass {
  position: [f32; 3],
  mass: f32,
  force: [f32; 3],
  // if a head-hair collision happened, a penalty force will add to the node
  penalty: [f32; 3],
}

#[derive(Component, Debug)]
struct Spring {
  stiffness: f32,
  damping: f32,
}

#[derive(Component, Debug)]
struct Hair {
  mass: [Mass, 20],
  spring: Spring,
}
```

And hair strands that make a head of hairs:

```rust
#[derive(Component, Debug)]
struct HairStrands {
  hair: [Hair, 40],
}
```

Then we create hairs:

```rust
let hair = world.create_entity().with(HairStrands { ... }).build();
```

### Mass-Spring System

First system is the head-hair collision system, add penalty force to mass nodes that are inside the head.

```rust
struct MassSpringSystem;

impl<'a> System<'a> for MassSpringSystem {
    type SystemData = WriteStorage<'a, HairStrands>;

    fn run(&mut self, data: Self::SystemData) {
        println!("{}", data.hair);
    }
}
```

Second system for hair is the system to update node position based on force on node.

```rust
struct MassSpringSystem;

impl<'a> System<'a> for MassSpringSystem {
    type SystemData = WriteStorage<'a, HairStrands>;

    fn setup(&mut self, res: &mut Resources) {
      use specs::prelude::SystemData;
      Self::SystemData::setup(res);
      // Get center of head mesh and radius of bonding box

      // randomly get points in upper half of ball, let them be first position of hair-mass-node
      // other hair-mass-node's position set to previous + {..., z: 10. }
      self.hair = ...;
    }

    fn run(&mut self, data: Self::SystemData) {

      println!("{}", data.hair);
    }
}
```

## Reference

1. http://www.fannieliu.com/hairsim/hairsim.html
