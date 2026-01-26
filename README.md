# Arbiter — Coming Soon 🚧

> **Note:** In this project, **Tx == Bundle**  
A *bundle* is a vector of transactions.  
A single transaction is treated as a bundle of length `1`.

---

## What is Arbiter?

**Arbiter** is a block engine clone.

---

## System Overview

Arbiter is split into 5 components:

1. **Tx Receive / Tx Send**
2. **Filtering Layer**
3. **Auction Batch (50ms)**
4. **gRPC Stream to Validator**
5. **Bank Freeze Simulation (Optional)**

---

## Components

### 1) Tx Receiving / Tx Sending

Incoming transactions are always treated as bundles:

- `bundle = Vec<Tx>`
- if user sends one tx → `bundle = vec![tx]`

#### API
`POST /send`

- Accepts `Vec<Tx>`
- **Bundle size limit:** `<= 5`

Example internal shape:
```rust
Vec<Tx> // 1..=5
```

---

### 2) Filtering Layer

Bundles go into a **pending ring buffer**.

Filtering rule: **do heavy work last**, and most filtering is deferred into auction stage.

Filter priority (light → heavy):

```
Blockhash Expiration  <<<<  Tip Filter  <<<  Signature Verify
```

✅ **Blockhash expiration check first** (cheap + fast)  
✅ **Tip-based filtering next**  
✅ **Signature verify last** (most expensive)

#### Pending Buffer

- A ring buffer stores incoming bundles until they are pulled by Auction.

---

### 3) Auction Batch (50ms Tick)

Auction operates on a **fixed internal time tick**.

Implementation idea:

- internal clock loop (`std::thread`)
- every **50ms**
  - pull everything from pending ring buffer
  - reset buffer counter
  - sort bundles by **decreasing TIP**
  - push sorted bundles into an outbound channel for validator stream

#### Output Shape
Auction produces a batch of prioritized bundles:

```rust
Vec<([Tx; 5], Tip)>
```

Notes:
- internally bundle may be padded into `[Tx;5]` form for transport
- sorting key = highest tip first

---

### 4) gRPC Streaming to Validator

This component handles:

- full **protobuf** implementation
- connect / reconnect / disconnect resilience
- stream outbound auction batches continuously

#### Core Responsibilities
- Maintain a long-lived gRPC stream
- Send `Vec<Vec<Tx>>` batches (**final format TBD**)
- Track validator state

#### Open Questions / TODO
- slot tracking: determine whether the connected validator is leader
- confirm batch payload format:
  - `Vec<Vec<Tx>>` vs `Vec<Bundle>`
  - whether to send only new batch or resend window

---

### 5) Bank Freeze Simulation (Optional)

Optional simulation executes bundles against a **frozen bank snapshot**.

#### Problem
At time `T`:

- `validator bank != rpc bank`
- RPC bank state can be delayed / stale

But simulation needs the **latest possible bank** for meaningful results.

#### Goal
Find the best way to simulate against the freshest bank source:
- ideally validator-side bank state
- avoid laggy RPC state

---

## Data Flow

High-level pipeline:

```
/send API
   ↓
Pending Ring Buffer
   ↓
Auction Tick (every 50ms)
   ↓  (sort by TIP desc)
Outbound Stream Channel
   ↓
gRPC Validator Stream
   ↓
(Optional) Bank Freeze Simulation
```

---

## Project Structure (Suggested)

```bash
arbiter/
├── crates/
│   ├── api/                 # HTTP API: /send
│   ├── buffer/              # ring buffer + pending bundles
│   ├── auction/             # 50ms tick + tip sorting + batching
│   ├── grpc-client/         # validator grpc stream + reconnect logic
│   ├── filters/             # blockhash/tip/sigverify modules
│   └── simulation/          # bank freeze + replay logic (optional)
├── proto/                   # protobuf definitions
├── configs/                 # config files (ports, limits, tick interval)
└── README.md
```

---

## Config Defaults (Draft)

| Setting | Default |
|--------|---------|
| Bundle size limit | `5` |
| Auction tick interval | `50ms` |
| Sort order | Tip descending |
| Pending buffer | Ring buffer |
| Filter ordering | blockhash → tip → sigverify |

---

## Milestones

### Phase 1: Intake + Queue
- [ ] `/send` API accepts `Vec<Tx>` bundles (1..=5)
- [ ] ring buffer for pending bundles

### Phase 2: Auction batching
- [ ] 50ms tick loop
- [ ] pull + reset pending buffer
- [ ] sort bundles by tip
- [ ] push to outbound stream channel

### Phase 3: gRPC streaming
- [ ] full protobuf schema
- [ ] connect/reconnect handling
- [ ] stream batches to validator

### Phase 4: Filtering correctness + performance
- [ ] enforce filter ordering
- [ ] move heavy verification late (auction stage)

### Phase 5: Simulation (optional)
- [ ] freeze bank snapshot
- [ ] replay tx bundles
- [ ] handle validator bank vs rpc bank drift

---

## Notes / Assumptions

- A **bundle is the atomic unit** (even 1 tx = bundle)
- heavy checks should not block ingestion
- prioritization is tip-based auction ordering
- validator streaming format is still evolving
