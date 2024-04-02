# Introduction
Rust io_uring examples, based on [@tokio/io_uring](https://github.com/tokio-rs/io-uring)

## Networking examples
- [tcp_simple](examples/tcp_simple): A simple echo server in TCP, using only basic functionality
- [uds_simple](examples/uds_simple): As above, but using Unix Domain Sockets
- [tcp_multishot](examples/tcp_multishot): As in `tcp_simple`, but with `multishot` support for accept and receive.
- [uds_multishot](examples/uds_multishot): As in `tcp_multishot`, but with Unix Domain Sockets. (**pending**)

## Storage examples
- [xfs_simple](examples/xfs_simple): Using `io_uring` to read a file from XFS (**not working**)
- [nvme](examples/nvme): Using `io_uring` to read a NVMe device

# TODOs:

- UDP example
- Fix `xfs_simple` example
- Better docs all around
- Use [ring mapped buffers](https://github.com/axboe/liburing/wiki/io_uring-and-networking-in-2023#provided-buffers)

# Questions

- Should I use fixed buffers with NVMe commands?
- How do I create a QUIC example?
- How do I render the commands async?
- How many worker threads?