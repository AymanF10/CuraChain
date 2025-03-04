```markdown
# 🏥 CuraChain - Decentralized Medical Crowdfunding Protocol on Solana  
*Empowering Transparent and Secure Medical Funding*

![CuraChain Banner](https://via.placeholder.com/1500x500.png?text=Secure+Medical+Fundraising+on+Solana)  
**A Solana-based protocol bridging patients in need with global donors through encrypted, auditable, and regulatory-compliant smart contracts.**

---

## 📜 Table of Contents
- [Mission](#-mission)
- [Core Features](#-core-features)
- [Technical Deep Dive](#-technical-deep-dive)
- [Architecture Diagram](#-architecture-diagram)
- [Tech Stack](#-tech-stack)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Local Deployment](#local-deployment)
- [Workflow Examples](#-workflow-examples)
- [Project Structure](#-project-structure)
- [Testing Suite](#-testing-suite)
- [Compliance & Security](#%EF%B8%8F-compliance--security)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)
- [Support](#-support)

---

## 🎯 Mission

CuraChain addresses the critical gap in accessible medical funding by leveraging blockchain technology to:
- Provide **end-to-end encrypted case submission** for patient privacy.
- Enable **trustless verification** by accredited medical professionals.
- Ensure **100% fund traceability** via Solana PDAs and on-chain escrow.
- Comply with **GDPR, OFAC, and global crowdfunding regulations**.

---

## 💡 Core Features

### 1. **Patient Case Management**
- **Encrypted Document Upload**: IPFS/Arweave links encrypted with verifier public keys.
- **Unique Case ID Generation**: Auto-incremented via `CaseCounter` PDA.
- **Real-Time Status Tracking**: 
  ```rust
  msg!("Case Status - ID: {}, Status: {:?}", case.case_id, case.status);
  ```
- **PDA Initialization**: 
  ```rust
  seeds = [b"patient-case", patient_pubkey]
  ```

### 2. **Verification Governance**
- **Whitelisting Mechanism**:
  ```rust
  #[account(seeds = [b"verifier", verifier_pubkey], bump)]
  pub verifier_registry: Account<'info, VerifierRegistry>
  ```
- **Time-Bound Voting**: 3-day window enforced via Solana Clock.
- **Threshold-Based Approval**:
  - 50% minimum verifier participation.
  - 70% approval rate for case acceptance.

### 3. **Escrow & Donation System**
- **Escrow PDA Creation**:
  ```rust
  #[account(seeds = [b"escrow", case_id], bump)]
  pub escrow: Account<'info, EscrowPDA>
  ```
- **Direct SOL Donations**: Tracked via `DonationPDA` with donor-specific seeds.
- **Fund Release Automation**: Triggered when target amount is met.

### 4. **Compliance Engine**
- **Sanction Screening**: OFAC-compliant donor checks via `CheckCompliance` instruction.
- **Data Sovereignty**: On-chain metadata + off-chain document storage (IPFS CID: `QmXYZ...`).

### 5. **Multi-Role Dashboards**
- **Patients**: Track donations, download audit reports.
- **Donors**: Filter cases by medical specialty/urgency.
- **Admins**: Real-time analytics via `GenerateReport` instruction.

---

## 🔍 Technical Deep Dive

### Smart Contract Modules
| Module               | Key Functions                          | PDAs Involved               |
|----------------------|----------------------------------------|-----------------------------|
| `submit_case`        | Encrypt links, initialize PatientCase | `PatientCase`, `CaseCounter`|
| `verify_case`        | Vote tallying, time checks            | `VerificationPDA`           |
| `create_escrow`      | Fund isolation for approved cases     | `EscrowPDA`                 |
| `donate`             | SOL transfer logic                    | `DonationPDA`               |

### Data Flow
1. Patient submits case → `PatientCase` PDA initialized.
2. Verifiers review → Votes recorded in `VerificationPDA`.
3. Case approved → `EscrowPDA` created.
4. Donations tracked → Funds auto-release to hospital upon goal.
5. Real-time updates via Solana transaction logs.

---

## 🖼️ Architecture Diagram

```
[Patient] → [Encrypted IPFS Link]
  ↓
[Solana Program]
  ├─ PatientCase PDA → Case Metadata
  ├─ EscrowPDA → Fund Pooling
  └─ VerificationPDA → Governance
        ↓
[Donors] → [Transparent Contributions]
```

---

## 🛠️ Tech Stack

**Blockchain**  
- Solana Mainnet (Anchor v0.28.0)  
- Program Derived Addresses (PDAs)

**Frontend** *(Future Phase)*  
- React + Next.js  
- Solana Wallet Adapter

**Storage**  
- IPFS (Document Storage)  
- Arweave (Immutable Backups)

**Security**  
- AES-256 Encryption  
- Anchor Security Constraints

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.65+
- Solana CLI 1.14.18+
- Node.js 18.x

### Installation
```bash
git clone https://github.com/curachain/core.git
cd core/programs/curachain
anchor build
```

### Local Deployment
1. Start Validator:
   ```bash
   solana-test-validator --reset
   ```
2. Deploy Program:
   ```bash
   anchor deploy --provider.cluster localnet
   ```
3. Run Tests:
   ```bash
   anchor test --skip-build
   ```

---

## 📂 Project Structure

```
curachain/
├── programs/
│   └── curachain/
│       ├── src/
│       │   ├── instructions/      # Core logic modules
│       │   │   ├── submit_case.rs
│       │   │   ├── verify_case.rs
│       │   │   └── donate.rs
│       │   ├── state/             # PDA structs
│       │   │   ├── patient_case.rs
│       │   │   └── escrow_pda.rs
│       │   ├── constants.rs       # Seeds & thresholds
│       │   └── lib.rs             # Program entry
├── tests/                         # TypeScript integration tests
├── target/idl/                   # Anchor IDL
└── apps/                         # Future UI
```

---

## 🔬 Testing Suite

**Key Test Cases** *(See `curachain.ts`)*:
1. `initialize_counter()`: Validate global counter PDA.
2. `submit_patient_case("ipfs://QmXYZ")`: Test encrypted link storage.
3. `whitelist_verifier()`: Add medical board to registry.
4. `finalize_verification()`: Timeout and threshold checks.

Run all tests:
```bash
anchor test --skip-build
```

---

## ⚖️ Compliance & Security

**Audits**  
- Pending third-party review (Contact: audits@curachain.org)

**Encryption**  
- Patient documents: AES-256 via off-chain key management.
- On-chain links: Base64 encoded IPFS CIDs.

**Regulatory Adherence**  
- GDPR: Right-to-be-forgotten via PDA closures.
- OFAC: Donor wallet screening in `CheckCompliance`.

---

## 🗺️ Roadmap

**Q4 2025**  
- NFT Donor Badges (ERC-1155 Compatibility)
- Cross-Chain Escrows (EVM ↔ Solana Bridges)

**Q1 2026**  
- AI-Pledged Case Prioritization
- Gasless Transactions via Compression

---

## 🤝 Contributing

1. Fork & clone the repo.
2. Create feature branch:
   ```bash
   git checkout -b feat/your-feature
   ```
3. Write tests for new instructions.
4. Submit PR with `[RFC]` tag for review.



---

## 📄 License  
MIT License - See [LICENSE](LICENSE).

---

## 🆘 Support  
**Team**: Ernest & Ayman (Co-Founders)  
**Email**: aymanf.gis@protonmail.com 
```

---

### 🔗 Additional Resources
- [Protocol Whitepaper](CuraChain-Protocol-Requirements.pdf)
- [User Stories & Personas](CuraChain-User-Stories.docx)
- [Audit Report Template](https://github.com/curachain/audits) *(Coming Soon)*