# 🏥 CuraChain - Decentralized Medical Crowdfunding Protocol on Solana  
*Empowering Transparent and Secure Medical Funding*

![CuraChain Banner](https://via.placeholder.com/1500x500.png?text=Secure+Medical+Fundraising+on+Solana)  
**A Solana-based protocol bridging patients in need with global donors through encrypted, auditable, and regulatory-compliant smart contracts.**

---

## 📜 Table of Contents
- [Mission](#-mission)
- [Core Features](#-core-features)
- [Technical Architecture](#-technical-architecture)
- [Smart Contract Modules](#-smart-contract-modules)
- [Tech Stack](#-tech-stack)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Local Development](#local-development)
  - [Testing](#testing)
- [Program Architecture](#-program-architecture)
- [Security & Compliance](#-security--compliance)
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
- Support both **SOL and SPL token** donations for maximum flexibility.
- Comply with **global crowdfunding regulations**.

---

## 💡 Core Features

### 1. **Patient Case Management**
- **Encrypted Medical Record Links**: Secure storage of sensitive patient data.
- **Unique Case ID Generation**: Automatic case numbering via `CaseCounter` PDA.
- **Status Tracking**: Real-time monitoring of verification and funding status.
- **SPL Token Support**: Accept donations in multiple tokens.

### 2. **Verification Governance**
- **Verifier Registry**: Whitelisted medical professionals with on-chain verification authority.
- **Voting Mechanism**: Threshold-based approval system (70% approval required).
- **Admin Override**: Safeguard mechanism for edge cases.

### 3. **Donation & Escrow System**
- **Multi-Token Support**: Accept donations in SOL and various SPL tokens.
- **Transparent Fund Tracking**: On-chain record of all donations.
- **Secure Fund Release**: Controlled disbursement to treatment providers.

### 4. **Administration**
- **Verifier Management**: Add/remove authorized medical verifiers.
- **Case Oversight**: Administrative capabilities for exceptional situations.
- **Fund Release Authorization**: Multi-signature approval for fund disbursement.

---

## 🔍 Technical Architecture

CuraChain is built on Solana using the Anchor framework, leveraging Program Derived Addresses (PDAs) to create a secure and efficient system:

### Key PDAs and Accounts

| Account Type | Purpose | Seeds |
|-------------|---------|-------|
| `Administrator` | Stores admin authority | `[b"admin"]` |
| `CaseCounter` | Tracks and assigns case IDs | `[b"case-counter"]` |
| `PatientCase` | Stores case details and verification status | `[b"patient-case", patient_pubkey]` |
| `CaseIDLookup` | Maps case IDs to patient accounts | `[b"case-id", case_id]` |
| `VerifiersList` | Registry of authorized medical verifiers | `[b"verifiers-list"]` |
| `Verifier` | Individual verifier account | `[b"verifier", verifier_pubkey]` |
| `DonorInfo` | Tracks donor contribution history | `[b"donor", donor_pubkey]` |

### Data Flow
1. **Case Submission**: Patient submits case with medical documentation.
2. **Verification**: Medical professionals review and vote on case legitimacy.
3. **Fundraising**: Approved cases receive donations in SOL and SPL tokens.
4. **Fund Release**: Verified treatment providers receive funds for patient care.

---

## 📦 Smart Contract Modules

| Module | Description | Key Functions |
|--------|-------------|--------------|
| `initialize_admin.rs` | Admin account setup | `initialize_administrator` |
| `verifiers_operations.rs` | Verifier management | `initialize_verifiers_list`, `add_verifier`, `remove_verifier` |
| `create_patient_case.rs` | Case creation | `submit_cases` |
| `verify_patient_case.rs` | Case verification | `verify_patient` |
| `donate_funds.rs` | Donation handling | `donate` |
| `initialize_patient_spl_account.rs` | SPL token support | `initialize_patient_spl_account` |
| `release_funds.rs` | Fund disbursement | `release_funds` |
| `admin_override_case.rs` | Admin controls | `admin_override_case` |
| `close_rejected_case.rs` | Case cleanup | `close_rejected_case` |

---

## 🛠️ Tech Stack

**Blockchain**  
- Solana Blockchain  
- Anchor Framework v0.31.1  
- Program Derived Addresses (PDAs)  
- SPL Token Standard

**Development**  
- Rust (for on-chain program)  
- TypeScript (for testing and client integration)  
- Mocha (for test framework)

**Security**  
- AES-GCM Encryption (for sensitive data)  
- Base64 Encoding

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ and Cargo
- Solana CLI 1.16.0+
- Node.js 18+ and Yarn
- Anchor CLI 0.31.1+

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/curachain.git
cd curachain
```

2. Install JavaScript dependencies:
```bash
yarn install
```

3. Build the Solana program:
```bash
anchor build
```

### Local Development

1. Start a local Solana validator:
```bash
solana-test-validator
```

2. Deploy the program to localnet:
```bash
anchor deploy
```

3. Update the program ID in `Anchor.toml` and `lib.rs` if needed.

### Testing

Run the test suite:
```bash
anchor test
```

---

## 📂 Program Architecture

```
curachain/
├── programs/
│   └── curachain/
│       ├── src/
│       │   ├── instructions/       # Core logic modules
│       │   │   ├── create_patient_case.rs
│       │   │   ├── verify_patient_case.rs
│       │   │   ├── donate_funds.rs
│       │   │   └── ...
│       │   ├── states/            # Account definitions & contexts
│       │   │   ├── accounts.rs
│       │   │   ├── contexts.rs
│       │   │   ├── errors.rs
│       │   │   └── ...
│       │   └── lib.rs             # Program entry point
├── tests/                         # TypeScript tests
│   └── curachain.ts
├── app/                          # Frontend (future)
└── Anchor.toml                   # Project configuration
```

---

## 🔒 Security & Compliance

**Security Features**  
- Encrypted medical record links  
- PDA-based access control  
- Threshold-based verification  
- Multi-signature fund release

**Compliance Considerations**  
- Patient data privacy protection  
- Transparent fund tracking  
- Verifiable donation history  
- Administrative oversight

---

## 🗺️ Roadmap

**Phase 1: Core Protocol (Current)**  
- Patient case submission and verification  
- SOL and SPL token donation support  
- Basic fund release mechanism

**Phase 2: Enhanced Features**  
- Mobile-friendly frontend interface  
- Multi-signature governance  
- Enhanced analytics dashboard  
- Integration with medical payment systems

**Phase 3: Ecosystem Expansion**  
- Cross-chain bridge support  
- Automated verification with trusted oracles  
- NFT-based donor recognition  
- Global regulatory compliance framework

---

## 🤝 Contributing

We welcome contributions to CuraChain! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Commit your changes: `git commit -m 'Add some feature'`
4. Push to the branch: `git push origin feature/your-feature-name`
5. Open a Pull Request

Please ensure your code follows our style guidelines and includes appropriate tests.

---

## 📄 License  
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🆘 Support  
**Team**: Ernest & Ayman (Co-Founders)  
**Email**: aymanf.gis@protonmail.com 

---

### 🔗 Additional Resources
- [Protocol Whitepaper](CuraChain-Protocol-Requirements.pdf)
- [User Stories & Personas](CuraChain-User-Stories.docx)
- [Audit Report Template](https://github.com/curachain/audits) *(Coming Soon)*