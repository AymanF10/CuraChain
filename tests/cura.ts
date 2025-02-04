import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Curachain } from "../target/types/cura";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert, expect } from "chai";

describe("Cura", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Curachain as Program<Curachain>;

  // Test accounts
  const admin = provider.wallet;
  const patientWallet = Keypair.generate();
  const donorWallet = Keypair.generate();
  const verifierWallet = Keypair.generate();
  const hospitalWallet = Keypair.generate();

  let tokenMint: PublicKey;
  let patientCasePda: PublicKey;
  let escrowTokenAccount: PublicKey;
  let donorTokenAccount: PublicKey;
  let hospitalTokenAccount: PublicKey;
  let verifierPda: PublicKey;

  const caseId = new anchor.BN(1);
  const medicalDescription = "Severe condition requiring surgery";
  const requiredFunds = new anchor.BN(1000);
  const medicalRecordsTxId = "QmXYZ123";
  const donationAmount = new anchor.BN(100);

  before(async () => {
    // Create a new SPL-Token mint
    const token = await Token.createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      6,
      TOKEN_PROGRAM_ID
    );
    tokenMint = token.publicKey;

    // Create token accounts for donor, escrow, and hospital
    donorTokenAccount = await token.createAccount(donorWallet.publicKey);
    escrowTokenAccount = await token.createAccount(provider.wallet.publicKey);
    hospitalTokenAccount = await token.createAccount(hospitalWallet.publicKey);

    // Mint tokens to the donor's token account
    await token.mintTo(donorTokenAccount, admin.payer, [], donationAmount.toNumber());

    // Derive PDAs
    [patientCasePda] = await PublicKey.findProgramAddress(
      [Buffer.from("patient_case"), patientWallet.publicKey.toBuffer()],
      program.programId
    );

    [verifierPda] = await PublicKey.findProgramAddress(
      [Buffer.from("verifier"), verifierWallet.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Creates a patient case", async () => {
    await program.methods
      .createPatientCase(
        caseId,
        medicalDescription,
        requiredFunds,
        medicalRecordsTxId,
        tokenMint
      )
      .accounts({
        patientCase: patientCasePda,
        patientWallet: patientWallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([patientWallet])
      .rpc();

    const patientCaseAccount = await program.account.patientCase.fetch(patientCasePda);
    assert.equal(patientCaseAccount.caseId.toString(), caseId.toString());
    assert.equal(patientCaseAccount.medicalDescription, medicalDescription);
    assert.equal(patientCaseAccount.requiredFunds.toString(), requiredFunds.toString());
    assert.equal(patientCaseAccount.verified, false);
    assert.equal(patientCaseAccount.tokenMint.toBase58(), tokenMint.toBase58());
  });

  it("Verifies a patient case", async () => {
    await program.methods
      .verifyCase()
      .accounts({
        patientCase: patientCasePda,
        verifier: verifierPda,
        verifierWallet: verifierWallet.publicKey,
      })
      .signers([verifierWallet])
      .rpc();

    const patientCaseAccount = await program.account.patientCase.fetch(patientCasePda);
    assert.equal(patientCaseAccount.verified_by.length, 1);
  });

  it("Creates a donation", async () => {
    await program.methods
      .createDonation(caseId, donationAmount)
      .accounts({
        patientCase: patientCasePda,
        donation: donorTokenAccount, 
        donorWallet: donorWallet.publicKey,
        donorTokenAccount: donorTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([donorWallet])
      .rpc();

    const escrowBalance = await provider.connection.getTokenAccountBalance(escrowTokenAccount);
    assert.equal(escrowBalance.value.amount, donationAmount.toString());
  });

  it("Withdraws funds to the hospital's wallet", async () => {
    await program.methods
      .withdrawFunds()
      .accounts({
        patientCase: patientCasePda,
        escrowTokenAccount: escrowTokenAccount,
        hospitalTokenAccount: hospitalTokenAccount,
        authority: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const hospitalBalance = await provider.connection.getTokenAccountBalance(hospitalTokenAccount);
    assert.equal(hospitalBalance.value.amount, donationAmount.toString());

    const escrowBalance = await provider.connection.getTokenAccountBalance(escrowTokenAccount);
    assert.equal(escrowBalance.value.amount, "0");
  });

  it("Fails to withdraw funds if not authorized", async () => {
    const unauthorizedWallet = Keypair.generate();

    await expect(
      program.methods
        .withdrawFunds()
        .accounts({
          patientCase: patientCasePda,
          escrowTokenAccount: escrowTokenAccount,
          hospitalTokenAccount: hospitalTokenAccount,
          authority: unauthorizedWallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([unauthorizedWallet])
        .rpc()
    ).to.be.rejected;
  });

  it("Fails to withdraw funds if case is not verified", async () => {
    const unverifiedCasePda = await PublicKey.findProgramAddress(
      [Buffer.from("patient_case"), Keypair.generate().publicKey.toBuffer()],
      program.programId
    );

    await expect(
      program.methods
        .withdrawFunds()
        .accounts({
          patientCase: unverifiedCasePda,
          escrowTokenAccount: escrowTokenAccount,
          hospitalTokenAccount: hospitalTokenAccount,
          authority: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc()
    ).to.be.rejected;
  });

  it("Fails to withdraw funds if escrow has insufficient funds", async () => {
    const emptyEscrowPda = await PublicKey.findProgramAddress(
      [Buffer.from("escrow"), new anchor.BN(2).toBuffer("le", 8)],
      program.programId
    );

    await expect(
      program.methods
        .withdrawFunds()
        .accounts({
          patientCase: patientCasePda,
          escrowTokenAccount: emptyEscrowPda,
          hospitalTokenAccount: hospitalTokenAccount,
          authority: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc()
    ).to.be.rejected;
  });
});