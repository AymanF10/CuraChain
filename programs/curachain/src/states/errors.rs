//use aes_gcm::aes::cipher::OverflowError;
use anchor_lang::prelude::*;


#[error_code]
pub enum CuraChainError{
    #[msg("Patient Case Has Not Been Verified")]
    NotVerifiedSuccessfully,

    #[msg("Patient Case Has Already Been Verified")]
    CaseAlreadyVerified,

    #[msg("Only Callable By Administrator")]
    OnlyAdmin,

    #[msg("Only Caller With The Verifier Role Can Call This Function")]
    OnlyVerifier,
    #[msg("Provided Admin Account Is Invalid")]
    InvalidAdminAccount,

    #[msg("Verifier Address Already Exist In The Registry Of Verifiers")]
    VerifierAlreadyExists,

    #[msg("Verifier Address Not Found In Registry")]
    VerifierNotFound,

    #[msg("Specified Verifier Address Does Not Exists")]
    InvalidVerifierAddress,

    #[msg("There Can Be Maximum Of 5 Multisig members")]
    TooManyMembers,

    #[msg("Specified Signer Already Part Of Multisig")]
    SignerAlreadyExists,

    #[msg("Specified Signer Is Not Part Of Multisig")]
    NotMultisigMember,

    #[msg("Specified Case ID Does Not Exist")]
    InvalidCaseID,

    #[msg("Verifier Can Only Vote Once On A Case")]
    VerifierAlreadyVoted,

    #[msg("Possible Overflow Error Detected")]
    OverflowError,

    #[msg("Possible Underflow Error Detected")]
    UnderflowError,

    #[msg("Escrow Account Creation For Patient Was Unsuccessful")]
    EscrowCreationFailed,

    #[msg("Escrow Account For Case Does Not Exist")]
    EscrowNotExist,

    #[msg("Escrow Account Verification With Passed Account Failed")]
    InvalidEscrowPDA,

    #[msg("Cannot Donate A Zero Amount")]
    NonZeroAmount,

    #[msg("Specified Token Does Not Match Donation Token")]
    TokenMismatched,

    #[msg("Donations Exceeds Total Needed Treatment Amount: Thank You")]
    DonationsExceeded,

    #[msg("Balance In Lamports Is Not Enough: Specify Lesser Amount")]
    InsufficientBalance,

    #[msg("No Donations Made To Specified Case")]
    NoDonationsMade,

    #[msg("Metadata Collection Does Not Match Specified Collection")]
    InvalidCollectionMint,

    #[msg("Balance In Lamports To Rent Account Is Not Sufficient")]
    InsufficientRentBalance,

    #[msg("Donations Cannot Be Made To Unverified Cases")]
    UnverifiedCase,

    #[msg("Patient Case Has Not Yet Been Verified")]
    CaseNotYetVerified,

    #[msg("Verifiers Can Still Do Verification On Case")]
    VerifiersVerificationActive,

    #[msg("Not Enough Verifiers Have Voted On The Case")]
    NotEnoughVerifiers,

    #[msg("No Transfer Proposal Initiated For Case")]
    NoProposalMade,

    #[msg("No Proposal With Such Index Exists")]
    InvalidProposalIndex,

    #[msg("Proposal Has Not Been Approved By Multisig")]
    ProposalNotApproved,

    #[msg("Proposal Has Already Been Executed")]
    ProposalAlreadyExecuted,

    #[msg("Multisig Member Has Already Voted On Case")]
    MultisigMemberVoted,

    #[msg("Lengths of Mints In Spl Donations And Remaining Accounts Differ")]
    InvalidMintsLength,

    #[msg("Created Facility Atas Donot Match")]
    MismatchedFacilityAtas,

    #[msg("Specified Mint Is Incorrect In Remaining Accounts")]
    InvalidRemainingMints,

    #[msg("Specified Patient Vault Is Incorrect In Remaining Accounts")]
    InvalidRemainingVaults,

    #[msg("Only Admin Or Multisig Member Can Initiate Funds Release")]
    UnauthorizedToTransfer,

    #[msg("Previous Metadata Info Is Invalid")]
    InvalidMetadata,

    #[msg("Derived Token Vaults Mismatch With Stored Token Vaults")]
    MismatchedTokenVaults,

    #[msg("The 70% Approval Threshold Was Passed")]
    CasePassedApproval,

    #[msg("Case Has Been Fully Funded: No Need For Further Donations")]
    CaseFullyFunded,
}