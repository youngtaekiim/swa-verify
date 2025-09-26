# gRPC mTLS Performance Benchmark

This project benchmarks the performance impact of mTLS on gRPC communication by comparing latency with and without TLS encryption.

## Project Structure

```
.
├── Cargo.toml                  # Workspace configuration
├── proto/
│   └── benchmark.proto         # Protocol buffer definition
├── benchmark-receiver/         # gRPC server
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/main.rs
├── benchmark-sender/           # gRPC client
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/main.rs
├── certs/                      # TLS certificates (generated)
├── generate_certs.sh           # Certificate generation script
└── README.md
```

## Setup

1. Generate TLS certificates:
```bash
./generate_certs.sh
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Running the Receiver

**Without TLS:**
```bash
cargo run --bin benchmark-receiver
```

**With TLS:**
```bash
cargo run --bin benchmark-receiver -- --tls
```

### Running the Sender

**Without TLS (100 requests):**
```bash
cargo run --bin benchmark-sender -- --requests 100
```

**With TLS (100 requests):**
```bash
cargo run --bin benchmark-sender -- --tls --requests 100
```

**Additional options:**
```bash
# With payload size and custom parameters
cargo run --bin benchmark-sender -- --tls --requests 1000 --delay 5 --payload medium

# Available payload sizes: small, medium, large
```

### Benchmark Example

1. Start receiver without TLS:
```bash
cargo run --bin benchmark-receiver
```

2. Run benchmark (in another terminal):
```bash
cargo run --bin benchmark-sender -- --requests 100
```

3. Stop receiver and start with TLS:
```bash
cargo run --bin benchmark-receiver -- --tls
```

4. Run benchmark with TLS:
```bash
cargo run --bin benchmark-sender -- --tls --requests 100
```

## Output

The sender outputs microsecond latencies for each request and provides summary statistics:

```
=== Benchmark Results ===
Total requests: 100
Successful: 100
Errors: 0
Min: 245μs
Max: 1205μs
Average: 387μs
Median: 356μs
95th percentile: 612μs
99th percentile: 892μs
```

## Features

- Microsecond precision timing
- Configurable request count and delay
- Variable payload sizes for testing
- TLS/non-TLS comparison
- Comprehensive statistics (min, max, avg, median, percentiles)
- Error handling and reporting