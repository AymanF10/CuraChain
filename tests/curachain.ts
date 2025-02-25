import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import { Curachain } from '../target/types/curachain';
import { assert, expect } from 'chai';

describe('Curachain Medical Funding Protocol', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Curachain as Program<Curachain>;
  const connection = provider.connection;

  // Test accounts
  const admin = anchor.web3.Keypair.generate();
  const patient = anchor.web3.Keypair.generate();
  const verifier = anchor.web3.Keypair.generate();
  const donor = anchor.web3.Keypair.generate();
  const trustedEntity = anchor.web3.Keypair.generate();

  // Program state PDAs
  let caseId: BN;
  let escrowPda: anchor.web3.PublicKey;
  let patientCasePda: anchor.web3.PublicKey;

  before(async () => {
    // Airdrop SOL to all test accounts
    const airdropAmount = anchor.web3.LAMPORTS_PER_SOL * 10;
    
    const confirmAirdrop = async (pubkey: anchor.web3.PublicKey) => {
      const tx = await connection.requestAirdrop(pubkey, airdropAmount);
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction({
        signature: tx,
        ...latestBlockhash
      });
    };
    console.log({
      admin: admin.publicKey.toString(), 
      patient: patient.publicKey.toString(),
      verifier: verifier.publicKey.toString(),
      donor: donor.publicKey.toString(),
    //escrowPda: escrowPda.toString(),
    //patientCasePda: patientCasePda.toString(),
    });

    await Promise.all([
      confirmAirdrop(admin.publicKey),
      confirmAirdrop(patient.publicKey),
      confirmAirdrop(verifier.publicKey),
      confirmAirdrop(donor.publicKey)
    ]);
 
  const adminBalance = await provider.connection.getBalance(admin.publicKey);
  const patientBalance = await provider.connection.getBalance(patient.publicKey);
  const verifierBalance = await provider.connection.getBalance(verifier.publicKey);
  const donorBalance = await provider.connection.getBalance(donor.publicKey);
  console.log("admin balance:", adminBalance);
  console.log("patient balance:", patientBalance);
  console.log("verifier balance:", verifierBalance);
  console.log("donor balance:", donorBalance);

});

  it('Should initialize the global case counter', async () => {
    const [counterPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("counter")],
      program.programId
    );

    await program.methods.initializeCounter()
      .accounts({
        caseCounter: counterPda,
        admin: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([admin])
      .rpc();

    const counter = await program.account.caseCounter.fetch(counterPda);
    assert.equal(counter.count.toNumber(), 0, "Counter should initialize to 0");
  });

  it('Should submit a new patient case', async () => {
    caseId = new BN(1);
    const [pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("patient-case"), patient.publicKey.toBytes()],
      program.programId
    );
    patientCasePda = pda;

    const encryptedLink = "ipfs://QmXYZ123encrypteddata";
    await program.methods.submitPatientCase(encryptedLink)
      .accounts({
        patientCase: patientCasePda,
        patient: patient.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([patient])
      .rpc();

    const caseAccount = await program.account.patientCase.fetch(patientCasePda);
    assert.equal(caseAccount.encryptedLink, encryptedLink);
    assert.equal(caseAccount.status.pending, true);
    assert.equal(caseAccount.caseId.toNumber(), 1);
  });

  it('Should whitelist a medical verifier', async () => {
    const [verifierPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("verifier"), verifier.publicKey.toBytes()],
      program.programId
    );

    await program.methods.whitelistVerifier({
      verifier: verifier.publicKey,
      verifierType: "Board Certified Doctor"
    })
      .accounts({
        verifierRegistry: verifierPda,
        admin: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([admin])
      .rpc();

    const registry = await program.account.verifierRegistry.fetch(verifierPda);
    assert.equal(registry.isVerified, true);
    assert.equal(registry.verifierType, "Board Certified Doctor");
  });

  it('Should verify a patient case', async () => {
    
    await program.methods.verifyCase({ approve: true })
      .accounts({
        patientCase: patientCasePda,
        verifier: verifier.publicKey,
      })
      .signers([verifier])
      .rpc();

    const caseAccount = await program.account.patientCase.fetch(patientCasePda);
    assert.equal(caseAccount.approveVotes.toNumber(), 1);
  });

  it('Should finalize verification after timeout', async () => {
    // Simulate time passing (3 secconds. Later will change it to 2 days in the main net)
    await new Promise(resolve => setTimeout(resolve, 3 * 1000));

    // Ensure caseId is passed as BN in an object
    await program.methods.finalizeVerification({ caseId })
      .accounts({
        patientCase: patientCasePda
      })
      .rpc();

    const caseAccount = await program.account.patientCase.fetch(patientCasePda);
    assert.equal(caseAccount.status.approved, true);
  });

  it('Should create an escrow for approved case', async () => {
    const [escrow] = anchor.web3.PublicKey.findProgramAddressSync(
      [new TextEncoder().encode("escrow"), caseId.toArrayLike(Buffer, 'le', 8)],
      program.programId
    );
    escrowPda = escrow;

    await program.methods.createEscrow({ caseId })
      .accounts({
        patientCase: patientCasePda,
        escrow: escrowPda,
        trustedEntity: trustedEntity.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([trustedEntity])
      .rpc();

    const escrowAccount = await program.account.escrowPda.fetch(escrowPda);
    assert.equal(escrowAccount.amount.toNumber(), 0);
    assert.equal(escrowAccount.caseId.toNumber(), 1);
  });

  it('Should process donations to escrow', async () => {
    const donationAmount = new BN(anchor.web3.LAMPORTS_PER_SOL); // 1 SOL
    const [donationPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        new TextEncoder().encode("donation"), 
        donor.publicKey.toBytes(), 
        escrowPda.toBytes()
      ],
      program.programId
    );

    await program.methods.donate({ amount: donationAmount })
      .accounts({
        escrow: escrowPda,
        donation: donationPda,
        donor: donor.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([donor])
      .rpc();

    const escrowAccount = await program.account.escrowPda.fetch(escrowPda);
    const donationAccount = await program.account.donationPda.fetch(donationPda);
    
    assert.equal(escrowAccount.amount.toNumber(), donationAmount.toNumber());
    assert.equal(donationAccount.amount.toNumber(), donationAmount.toNumber());
  });

  it('Should generate a funding report', async () => {
    const transactionSignature = await program.methods.generateReport()
      .accounts({
        patientCase: patientCasePda,
        escrow: escrowPda
      })
      .rpc();

    const transaction = await connection.getTransaction(transactionSignature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0
    });

    if (!transaction?.meta?.logMessages) {
      throw new Error("No transaction logs found");
    }

    const logs = transaction.meta.logMessages.join('\n');
    assert.include(logs, "Case Report");
    assert.include(logs, "Approved");
    assert.include(logs, "1000000000"); // 1 SOL in lamports
  });

  it('Should check donor compliance', async () => {
  
    const isCompliant = await program.methods.checkCompliance({})
      .accounts({
        donor: donor.publicKey
      })
      .view();

    assert.equal(isCompliant, true);
  });
});