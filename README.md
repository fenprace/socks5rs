# socks5rs

A simple socks5 server, written in Rust.

```
cargo run
```

The server is listening on `127.0.0.1:54000`.

## Limitations

- Authentication is not implemented.
- IPv6 is not supported.
- Bind & UDP associate is not supported.
- Lots of unhandled `unwrap()`s.

## References

- [RFC 1928](https://datatracker.ietf.org/doc/html/rfc1928)
- [《SOCKS 5 协议中文文档》](https://www.quarkay.com/code/383/socks5-protocol-rfc-chinese-traslation)
- [《使用 Rust 实现一个 Sock5 代理》](https://jimages.net/archives/269)
