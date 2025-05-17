# CuraChain

## A Decentralized Medical Crowdfunding Protocol on Solana

CuraChain is a blockchain-based platform that enables transparent and secure medical crowdfunding built on the Solana ecosystem. It connects patients in need with donors and ensures transparency through a robust verification system.

## Core Features

### Patient Case Submission
- Patients can submit medical cases with detailed descriptions
- Each case is assigned a unique identifier (e.g., "CASE0001")
- Cases include funding goals, medical documentation links, and descriptions

### Multi-layer Verification System
- A network of trusted medical verifiers evaluates patient cases
- 70% quorum required for case verification
- Verifiers must vote within a 10-day timeframe
- Admin override capability when verifiers don't respond within the timeframe

### Secure Fund Management
- All verified cases receive an escrow account
- Donors can contribute both SOL and SPL tokens
- Multisig requirement for fund disbursement (admin + verifiers)
- Donation tracking for transparency

### NFT Receipts
- Donors receive NFT receipts as proof of contribution
- NFTs include metadata about the donation amount and recipient case
- Enhances donor engagement and provides verifiable proof of donation

## Technical Architecture

### Smart Contract Design
- Built using Anchor framework on Solana
- Implements PDA (Program Derived Addresses) for secure account management
- Role-based permission system (admin, verifiers, patients, donors)
- Time-based verification windows with admin failsafe

### Key Accounts
1. **Administrator** - Manages the platform, verifiers, and can override verification
2. **Verifiers** - Trusted entities that validate patient cases
3. **Patient Cases** - Stores medical information and funding needs
4. **Escrow Accounts** - Secure holding for donated funds
5. **Donor Records** - Tracks all contributions
6. **NFT Metadata** - Stores donation receipt information

### Verification Workflow
1. Patient submits a case
2. Verifiers have 10 days to review and vote
3. Upon 70% positive votes, case is verified and escrow is created
4. If verification period expires, admin can approve or reject the case
5. Rejected cases can be closed by any user

### Fund Management
- Escrow accounts hold all donations (SOL and SPL tokens)
- Multisig authorization required for fund release
- Funds can only be donated to verified cases

## Security Features

- Time-locked verification to prevent infinite pending cases
- Admin override with mandatory time delay (10 days)
- Multisig requirement for fund disbursement
- Verifier quorum threshold (70%)
- Protection against duplicate voting
- Cannot donate to unverified cases
- Immutable record of verified cases

## Use Cases

### For Patients
- Access to global funding pool
- Transparent verification process
- Reduced fraud risk compared to traditional platforms

### For Donors
- Confidence in case validity through medical verification
- NFT receipts as proof of donation
- Ability to track fund usage

### For Medical Facilities
- Direct fund disbursement to approved healthcare providers
- Reduced administrative overhead
- Transparent payment rails

## Test Coverage

The protocol includes comprehensive tests for all key functionality:
- Administrator management
- Verifier addition and removal
- Patient case submission
- Case verification process
- Admin override capabilities
- Donation mechanics (SOL and SPL tokens)
- Case closure/rejection
- NFT receipt generation

## Future Enhancements

- Enhanced analytics dashboard for donation tracking
- Integration with traditional healthcare payment systems
- Mobile application for easier case submission and donation
- Expanded NFT utilities for donors
- Cross-chain bridges for accepting multiple cryptocurrencies

## Getting Started

### Prerequisites
- Solana CLI tools
- Node.js and npm/yarn
- Anchor framework

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/curachain.git
cd curachain

# Install dependencies
yarn install

# Build the program
anchor build

# Test the program
anchor test
```

### Deployment

```bash
# Deploy to Solana devnet
anchor deploy --provider.cluster devnet

# Initialize the program with admin
# (See documentation for detailed deployment steps)
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 