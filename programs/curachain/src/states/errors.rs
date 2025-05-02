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

    #[msg("Key Generation Was Not Successful")]
    KeyGenerationError,

    #[msg("Encryption of the link Is Unsuccessful!")]
    EncryptionError,

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

    #[msg("Donations Exceeds Total Needed Treatment Amount: Thank You")]
    DonationsExceeded,

    #[msg("Balance In Lamports Is Not Enough: Specify Lesser Amount")]
    InsufficientBalance,

    #[msg("Balance In Lamports To Rent Account Is Not Sufficient")]
    InsufficientRentBalance,

    #[msg("Donations Cannot Be Made To Unverified Cases")]
    UnverifiedCase,

    #[msg("Patient Case Has Not Yet Been Verified")]
    CaseNotYetVerified,

    #[msg("Not Enough Verifiers Have Voted On The Case")]
    NotEnoughVerifiers,

    #[msg("The 70% Approval Threshold Was Passed")]
    CasePassedApproval,

    #[msg("Case Has Been Fully Funded: No Need For Further Donations")]
    CaseFullyFunded,

    #[msg("Verifier voting period has expired")]
    VotingPeriodExpired,
}