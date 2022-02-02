# rescue-blanket -- escape values while they are being formatted

This crate provides `Escaped`, a wrapper for escaping special characters and
constructs in values while formatting them, and `Escaper`, a trait for defining
escaping logic. In addition, it provides `Escapable`, an augmentation trait for
facilitating wrapping values in `Escaped`.

The wrapping approach allows escaping arbitrary values implementing `Display`
without the need to buffer them.

## Example

```rust
use rescue_blanket::Escapable;
println!("foo=\"{}\"", "bar=\"baz\"".escaped_with(char::escape_default));
```

## License

This work is provided under the MIT license. See `LICENSE` for more details.

