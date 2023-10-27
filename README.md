# Ointer

Using the first bit, called `orientation bit`, of pointer data to store an extra boolean value, this crate wraps `Box/Rc(Weak)/Arc(Weak)` as `Ox/Oc(Ok)/Orc(Oak)`. An ointer can be used to represent geometric object together with its orientation. As an example, see code below: 
```rust, no_run
    // Construct a triangle
    let triangle = Orc::new(my_trangle_params...);
    // Clone and flip this triangle
    let triangle_flipped = triangle.clone_and_flip();
    // Here we got one triangle with its two references that have opposite normal directions
```