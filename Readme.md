# 🏥 CuraChain - Decentralized Medical Crowdfunding Protocol on Solana


---

## 📜 Table of Contents
- [Mission](#-mission)
- [Core Features](#-core-features)
- [Technical Architecture](#-technical-architecture)
  - [System Overview](#system-overview)
  - [Account Structure](#account-structure)
  - [Program Flow](#program-flow)
  - [Security Model](#security-model)
- [Smart Contract Modules](#-smart-contract-modules)
  - [Administration Module](#administration-module)
  - [Patient Case Module](#patient-case-module)
  - [Verification Module](#verification-module)
  - [Donation Module](#donation-module)
  - [Fund Release Module](#fund-release-module)
- [Tech Stack](#-tech-stack)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Local Development](#local-development)
  - [Testing](#testing)
- [Program Architecture](#-program-architecture)
  - [Directory Structure](#directory-structure)
  - [Key Components](#key-components)
- [Security & Compliance](#-security--compliance)
  - [Security Features](#security-features)
  - [Compliance Considerations](#compliance-considerations)
  - [Audit Status](#audit-status)
- [Contributing](#-contributing)
- [License](#-license)
- [Support](#-support)

---

## 🎯 Mission

CuraChain addresses the critical gap in accessible medical funding by leveraging blockchain technology to create a transparent, secure, and efficient crowdfunding platform. Our protocol aims to:

- Provide **end-to-end encrypted case submission** for patient privacy protection
- Enable **trustless verification** by accredited medical professionals through decentralized governance
- Ensure **100% fund traceability** via Solana PDAs and on-chain escrow mechanisms
- Support both **SOL and SPL token donations** for maximum flexibility and accessibility
- Comply with **global crowdfunding regulations** while maintaining transparency
- Eliminate intermediaries to reduce costs and increase fund efficiency

---

## 💡 Core Features

### 1. **Patient Case Management**



- **Encrypted Medical Record Links**: Secure storage of sensitive patient data using AES-GCM encryption
- **Unique Case ID Generation**: Automatic case numbering via `CaseCounter` PDA with collision-free ID assignment
- **Status Tracking**: Real-time monitoring of verification status, funding progress, and case lifecycle
- **SPL Token Support**: Accept donations in multiple tokens with per-token accounting
- **Case Metadata Storage**: On-chain storage of essential case information with privacy-preserving design

**Key Accounts:**
```
PatientCase {
    patient_pubkey: Pubkey,
    case_description: String,
    total_amount_needed: u64,
    total_raised: u64,
    case_id: String,
    verification_yes_votes: u8,
    voted_verifiers: Vec<Pubkey>,
    verification_no_votes: u8,
    is_verified: bool,
    patient_case_bump: u8,
    case_funded: bool,
    link_to_records: String,
    submission_timestamp: i64,
    spl_donations: Vec<SplDonationRecord>,
}
```

### 2. **Verification Governance**



- **Verifier Registry**: Whitelisted medical professionals with on-chain verification authority
- **Voting Mechanism**: Threshold-based approval system requiring 70% approval for case verification
- **Quorum Requirements**: Minimum 50% verifier participation for valid voting outcomes
- **Admin Override**: Safeguard mechanism for edge cases and dispute resolution
- **Time-Bound Voting**: Structured voting periods with expiration controls

**Verification Process:**
1. Administrator whitelists qualified medical professionals as verifiers
2. Patient submits case with medical documentation
3. Verifiers review documentation and cast votes (approve/reject)
4. When sufficient votes are cast, case status is automatically updated
5. Only verified cases can receive donations

### 3. **Donation & Escrow System**



- **Multi-Token Support**: Accept donations in SOL and various SPL tokens
- **Transparent Fund Tracking**: On-chain record of all donations with donor information
- **Escrow Mechanism**: Secure holding of funds until verification and disbursement criteria are met
- **Donation Caps**: Automatic donation limiting when funding goal is reached
- **Donor Analytics**: Tracking of donation history and contribution metrics

**Donation Flow:**
```
┌─────────┐     ┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│  Donor  │────▶│ Donate SOL/ │────▶│ Patient Case │────▶│ Fund Treatment │
│ Wallet  │     │  SPL Token  │     │   Escrow     │     │    Provider    │
└─────────┘     └─────────────┘     └──────────────┘     └────────────────┘
```

### 4. **Administration**



- **Verifier Management**: Add/remove authorized medical verifiers to the registry
- **Case Oversight**: Administrative capabilities for exceptional situations and dispute resolution
- **Fund Release Authorization**: Multi-signature approval for fund disbursement to treatment providers
- **System Configuration**: Parameter management for voting thresholds and system behavior

**Admin Operations:**
- Initialize administrator account
- Manage verifier registry
- Override case verification in exceptional circumstances
- Configure system parameters

---

## 🔍 Technical Architecture

### System Overview



CuraChain is built on Solana using the Anchor framework, leveraging Program Derived Addresses (PDAs) to create a secure and efficient system. The architecture consists of several interconnected components:

1. **On-Chain Program**: Core Solana program containing all business logic
2. **Account Structure**: PDAs and account definitions for data storage
3. **Client Interface**: JavaScript/TypeScript libraries for interacting with the protocol
4. **External Storage**: IPFS/Arweave integration for medical documentation (off-chain)

### Account Structure



#### Key PDAs and Accounts

| Account Type | Purpose | Seeds | Key Fields |
|-------------|---------|-------|------------|
| `Administrator` | Stores admin authority | `[b"admin"]` | `admin_pubkey`, `is_active`, `bump` |
| `CaseCounter` | Tracks and assigns case IDs | `[b"case-counter"]` | `current_id`, `counter_bump` |
| `PatientCase` | Stores case details and verification status | `[b"patient-case", patient_pubkey]` | `case_id`, `total_amount_needed`, `is_verified`, etc. |
| `CaseIDLookup` | Maps case IDs to patient accounts | `[b"case-id", case_id]` | `case_id_in_lookup`, `patient_pda`, `patient_address` |
| `VerifiersList` | Registry of authorized medical verifiers | `[b"verifiers-list"]` | `all_verifiers`, `verifier_registry_bump` |
| `Verifier` | Individual verifier account | `[b"verifier", verifier_pubkey]` | `verifier_key`, `is_verifier`, `verifier_bump` |
| `DonorInfo` | Tracks donor contribution history | `[b"donor", donor_pubkey]` | `donor_address`, `total_donations`, `donor_bump` |

### Program Flow



#### Patient Case Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌───────────────┐     ┌───────────────┐     ┌──────────────┐
│ Case        │     │ Verification │     │ Fundraising   │     │ Goal          │     │ Fund         │
│ Submission  │────▶│ Process      │────▶│ Period        │────▶│ Achievement   │────▶│ Disbursement │
└─────────────┘     └─────────────┘     └───────────────┘     └───────────────┘     └──────────────┘
```

1. **Case Submission**: Patient submits case with medical documentation
   - Creates `PatientCase` PDA
   - Assigns unique case ID from `CaseCounter`
   - Creates `CaseIDLookup` for efficient retrieval

2. **Verification**: Medical professionals review and vote on case legitimacy
   - Verifiers cast votes (yes/no)
   - System tracks voting progress
   - Case is marked verified when threshold is reached

3. **Fundraising**: Approved cases receive donations in SOL and SPL tokens
   - Donors contribute funds to case escrow
   - System tracks donation progress
   - Prevents over-funding of cases

4. **Fund Release**: Verified treatment providers receive funds for patient care
   - Administrator authorizes fund release
   - Funds transfer to treatment provider
   - Case is marked as completed

### Security Model



CuraChain implements multiple security layers to protect funds, data, and system integrity:

1. **Access Control**:
   - Role-based permissions (admin, verifier, patient, donor)
   - PDA-based ownership verification
   - Signature verification for all sensitive operations

2. **Data Protection**:
   - Encrypted medical record links
   - Privacy-preserving on-chain data
   - Secure off-chain storage for sensitive documents

3. **Fund Security**:
   - Escrow-based fund management
   - Multi-signature release requirements
   - Automatic donation caps

4. **System Integrity**:
   - Threshold-based verification system
   - Admin override capabilities for edge cases
   - Error handling and recovery mechanisms

---

## 📦 Smart Contract Modules

### Administration Module


| Component | File | Description | Key Functions |
|-----------|------|-------------|--------------|
| Admin Setup | `initialize_admin.rs` | Initializes the administrator account | `initialize_administrator` |
| Verifier Management | `verifiers_operations.rs` | Manages the verifier registry | `initialize_verifiers_list`, `add_verifier`, `remove_verifier` |
| Case Override | `admin_override_case.rs` | Provides admin override capabilities | `admin_override_case` |

**Key Functions:**
- `initialize_administrator`: Creates the admin PDA and sets initial permissions
- `initialize_global_verifiers_list_and_case_counter`: Sets up the verifier registry
- `add_or_remove_verifier`: Manages verifier whitelist
- `admin_override_case`: Allows admin intervention for exceptional cases

### Patient Case Module



| Component | File | Description | Key Functions |
|-----------|------|-------------|--------------|
| Case Creation | `create_patient_case.rs` | Handles patient case submission | `submit_cases` |
| Case Viewing | `view_case_details.rs` | Retrieves case information | `view_case_details` |
| Case Cleanup | `close_rejected_case.rs` | Handles rejected case cleanup | `close_rejected_case` |

**Key Functions:**
- `submit_cases`: Creates new patient case with medical information
- `view_case_details`: Retrieves case information for display
- `close_rejected_case`: Cleans up cases that fail verification

### Verification Module



| Component | File | Description | Key Functions |
|-----------|------|-------------|--------------|
| Case Verification | `verify_patient_case.rs` | Handles verification voting | `verify_patient` |

**Key Functions:**
- `verify_patient`: Records verifier votes on patient cases
- Implements voting threshold logic (70% approval required)
- Tracks verifier participation to ensure quorum

### Donation Module



| Component | File | Description | Key Functions |
|-----------|------|-------------|--------------|
| SOL Donation | `donate_funds.rs` | Handles SOL donations | `donate` |
| SPL Token Setup | `initialize_patient_spl_account.rs` | Sets up SPL token accounts | `initialize_patient_spl_account` |

**Key Functions:**
- `donate`: Processes donations to patient cases
- `initialize_patient_spl_account`: Creates token accounts for SPL donations

### Fund Release Module



| Component | File | Description | Key Functions |
|-----------|------|-------------|--------------|
| Fund Release | `release_funds.rs` | Handles fund disbursement | `release_funds` |

**Key Functions:**
- `release_funds`: Transfers funds from escrow to treatment provider
- Implements authorization checks and fund release conditions

---

## 🛠️ Tech Stack



### Blockchain Infrastructure
- **Solana Blockchain**: High-performance L1 blockchain with low transaction costs
- **Anchor Framework v0.31.1**: Solana development framework for safe smart contract development
- **Program Derived Addresses (PDAs)**: For secure account management and ownership
- **SPL Token Standard**: For multi-token support and management

### Development Technologies
- **Rust**: Core programming language for on-chain program development
  - Uses AES-GCM for encryption
  - Base64 for encoding
  - Anchor macros for program structure
- **TypeScript**: For testing and client integration
  - Anchor client libraries for program interaction
  - Mocha for test framework
  - Chai for assertions

### Development Tools
- **Solana CLI**: For deployment and chain interaction
- **Anchor CLI**: For building, testing, and deploying
- **TypeScript/JavaScript**: For client integration and testing

### Security Components
- **AES-GCM Encryption**: For sensitive data protection
- **Base64 Encoding**: For data transmission and storage
- **PDA-based Access Control**: For secure ownership and permissions

---

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** and Cargo
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Solana CLI 1.16.0+**
  ```bash
  sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"
  ```

- **Node.js 18+** and Yarn
  ```bash
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
  nvm install 18
  npm install -g yarn
  ```

- **Anchor CLI 0.31.1+**
  ```bash
  cargo install --git https://github.com/coral-xyz/anchor avm --locked
  avm install 0.31.1
  avm use 0.31.1
  ```

### Installation



1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/curachain.git
   cd curachain
   ```

2. **Install JavaScript dependencies:**
   ```bash
   yarn install
   ```

3. **Build the Solana program:**
   ```bash
   anchor build
   ```

4. **Update Program ID:**
   
   After building, update the program ID in `Anchor.toml` and `lib.rs` with the generated program ID:
   ```bash
   solana address -k target/deploy/curachain-keypair.json
   ```

### Local Development


1. **Start a local Solana validator:**
   ```bash
   solana-test-validator
   ```

2. **Deploy the program to localnet:**
   ```bash
   anchor deploy
   ```

3. **Initialize the program:**
   ```bash
   # Example initialization script
   ts-node scripts/initialize.ts
   ```

### Testing



Run the comprehensive test suite:
```bash
anchor test
```

The test suite covers:
- Administrator initialization and management
- Verifier registry operations
- Patient case submission and verification
- Donation processing (SOL and SPL tokens)
- Fund release mechanisms
- Error handling and edge cases

**Key Test Scenarios:**
1. Complete patient case lifecycle from submission to fund release
2. Verification voting with various threshold scenarios
3. Multi-token donation handling
4. Administrative override operations
5. Security constraint validation

---

## 📂 Program Architecture

### Directory Structure

```
curachain/
├── programs/
│   └── curachain/
│       ├── src/
│       │   ├── instructions/       # Core logic modules
│       │   │   ├── create_patient_case.rs
│       │   │   ├── verify_patient_case.rs
│       │   │   ├── donate_funds.rs
│       │   │   ├── release_funds.rs
│       │   │   ├── initialize_admin.rs
│       │   │   ├── verifiers_operations.rs
│       │   │   ├── admin_override_case.rs
│       │   │   ├── close_rejected_case.rs
│       │   │   ├── initialize_patient_spl_account.rs
│       │   │   ├── view_case_details.rs
│       │   │   └── mod.rs
│       │   ├── states/            # Account definitions & contexts
│       │   │   ├── accounts.rs     # Account structures
│       │   │   ├── contexts.rs     # Instruction contexts
│       │   │   ├── errors.rs       # Error definitions
│       │   │   ├── events.rs       # Event definitions
│       │   │   ├── constants.rs    # System constants
│       │   │   └── mod.rs
│       │   └── lib.rs             # Program entry point
├── tests/                         # TypeScript tests
│   └── curachain.ts               # Integration tests
├── app/                          # Frontend (future)
├── Anchor.toml                   # Project configuration
├── Cargo.toml                    # Rust dependencies
├── package.json                  # JavaScript dependencies
└── tsconfig.json                 # TypeScript configuration
```

### Key Components



#### 1. Program Entry Point (`lib.rs`)
- Defines program ID and instruction handlers
- Routes instructions to appropriate modules
- Implements cross-cutting validation

#### 2. Account Definitions (`states/accounts.rs`)
- Defines all on-chain data structures
- Implements account initialization space requirements
- Provides helper methods for account operations

#### 3. Instruction Contexts (`states/contexts.rs`)
- Defines input and account validation for each instruction
- Specifies required signers and account relationships
- Implements PDA derivation and constraint checking

#### 4. Error Handling (`states/errors.rs`)
- Defines custom error types for precise error reporting
- Implements error codes and messages
- Provides structured error handling throughout the program

#### 5. Event Emission (`states/events.rs`)
- Defines event structures for program activity logging
- Implements event emission throughout instruction handlers
- Facilitates off-chain monitoring and analytics

#### 6. Instruction Handlers (Various modules)
- Implement core business logic for each operation
- Enforce security constraints and access controls
- Manage state transitions and data validation

---

## 🔒 Security & Compliance

### Security Features



#### Access Control System
- **Role-Based Permissions**: Different capabilities for administrators, verifiers, patients, and donors
- **PDA Ownership**: Secure account ownership through program-derived addresses
- **Signature Verification**: Cryptographic verification of all operations

#### Data Protection
- **Encrypted Medical Records**: Sensitive patient data stored securely off-chain
- **Minimal On-Chain Data**: Only essential information stored on-chain
- **Secure Link Management**: Protected access to medical documentation

#### Fund Security
- **Escrow-Based Management**: Funds held in secure program-controlled accounts
- **Controlled Release**: Multi-step authorization for fund disbursement
- **Donation Caps**: Prevention of over-funding and fund misuse

### Compliance Considerations



#### Patient Privacy
- Medical records stored securely off-chain
- Minimal identifiable information on-chain
- Access controls for sensitive data

#### Financial Transparency
- Complete on-chain record of all donations
- Traceable fund flow from donor to recipient
- Auditable verification process

#### Regulatory Alignment
- Designed with consideration for healthcare fundraising regulations
- Transparent governance and fund management
- Verifiable case verification process

### Audit Status

- Security review planned for Q3 2023
- Code review and static analysis completed
- Formal verification pending

---

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🆘 Support

**Team**: Ernest & Ayman (Co-Founders)  
**Email**: aymanf.gis@protonmail.com  
**Discord**: [Join our community](https://discord.gg/curachain)  
**Twitter**: [@CuraChain](https://twitter.com/curachain)

---

### 🔗 Additional Resources

- [Protocol Whitepaper](CuraChain-Protocol-Requirements.pdf)
- [User Stories & Personas](CuraChain-User-Stories.docx)
- [Audit Report Template](https://github.com/curachain/audits) *(Coming Soon)*
- [API Documentation](https://docs.curachain.io) *(Coming Soon)*