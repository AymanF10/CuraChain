import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import { Curachain } from '../target/types/curachain';
import { assert, expect } from 'chai';
import { SystemProgram } from '@solana/web3.js';

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
  let caseCounterPda: anchor.web3.PublicKey;
  let escrowPda: anchor.web3.PublicKey;
  let patientCasePda: anchor.web3.PublicKey;
  //let caseId: BN; 
  //const case: number = 1456;
  const caseId = new anchor.BN(1456);

  before(async () => {

    [caseCounterPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      program.programId
    );
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
      confirmAirdrop(donor.publicKey),
      confirmAirdrop(trustedEntity.publicKey)
    ]);
  

  const adminBalance = await provider.connection.getBalance(admin.publicKey);
  const patientBalance = await provider.connection.getBalance(patient.publicKey);
  const verifierBalance = await provider.connection.getBalance(verifier.publicKey);
  const donorBalance = await provider.connection.getBalance(donor.publicKey);
  const trustedEntityBalance= await provider.connection.getBalance(trustedEntity.publicKey);
  console.log("admin balance:", adminBalance);
  console.log("patient balance:", patientBalance);
  console.log("verifier balance:", verifierBalance);
  console.log("donor balance:", donorBalance);
  console.log("trusted entity:", trustedEntityBalance);
});

it('Should initialize the global case counter', async () => {

 await program.methods.initializeCounter()
    .accounts({
      caseCounter: caseCounterPda,
      admin: admin.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([admin])
    .rpc();

  const counter = await program.account.caseCounter.fetch(caseCounterPda);
  assert.equal(counter.count.toNumber(), 0);
  assert.ok(counter.admin.equals(admin.publicKey));
});


it('Should submit a new patient case', async () => {
  // Get initial counter state
  const initialCounter = await program.account.caseCounter.fetch(caseCounterPda);
  const initialCount = new BN(initialCounter.count.toString());

  // Generate PDA for patient case (matches program's seed logic)
 [patientCasePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("patient-case"), patient.publicKey.toBytes()],
      program.programId
  );

  const encryptedLink = "ipfs://QmXYZ123encrypteddata";
  
  // Execute instruction
  await program.methods.submitPatientCase(encryptedLink,caseId)
      .accounts({
          patientCase: patientCasePda,
          caseCounter: caseCounterPda,
          patient: patient.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([patient])
      .rpc();

  // Verify results
  const caseAccount = await program.account.patientCase.fetch(patientCasePda);
  assert.equal(caseAccount.totalVerifiers.toNumber(), 1); 
  const updatedCounter = await program.account.caseCounter.fetch(caseCounterPda);

  // 1. Check encrypted link
  assert.equal(caseAccount.encryptedLink, encryptedLink);
  
  // 2. Proper status check for Anchor enum
  assert.deepEqual(caseAccount.status, { pending: {} });
  
  // 3. Verify case ID matches initial counter value
  assert.equal(
      caseAccount.caseId.toString(),
      initialCount.toString(),
      "Case ID should match initial counter value"
  );
  // 4. Verify counter incremented using BN operations
  assert.equal(
      updatedCounter.count.toString(),
      initialCount.add(new BN(1)).toString(),
      "Counter should increment by 1"
  );
  console.log("caseId:", caseId);
  console.log("initialcount:", initialCount);
});

  it('Should whitelist a medical verifier', async () => {
    const [verifierPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("verifier"), verifier.publicKey.toBytes()],
    program.programId
  );


    await program.methods.whitelistVerifier(verifier.publicKey,"Board Certified Doctor")
     .accounts({
        verifierRegistry: verifierPda,
        caseCounter: caseCounterPda,
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
    // 1. Get verifier registry
    const [verifierRegistryPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("verifier"), verifier.publicKey.toBytes()],
        program.programId
    );

    // 2. Generate verification PDA (matches program seeds)
    const [verificationPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from("verification"),
            patientCasePda.toBytes(),
            verifier.publicKey.toBytes()
        ],
        program.programId
    );

    // 3. Get initial case state
    const initialCase = await program.account.patientCase.fetch(patientCasePda);

    await program.methods.verifyCase(true)
        .accounts({
            patientCase: patientCasePda,
            verifierRegistry: verifierRegistryPda,
            verifier: verifier.publicKey,
            verificationPda: verificationPda,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .signers([verifier])
        .rpc();

    // 4. Verify updates
    const updatedCase = await program.account.patientCase.fetch(patientCasePda);
    const verificationAccount = await program.account.verificationPda.fetch(verificationPda);

    // Assertions
    assert.equal(
        updatedCase.approveVotes.toString(),
        initialCase.approveVotes.add(new BN(1)).toString(),
        "Approve votes should increment"
    );
    
    assert.isTrue(verificationAccount.vote, "Vote should be recorded as true");
    assert.equal(
        verificationAccount.caseId.toString(),
        updatedCase.caseId.toString(),
        "Case ID mismatch in verification PDA"
    );
    console.log("caseId:", caseId);
});

it('Should finalize verification after timeout', async () => {
  // Sleep for verification period + 1 second
  const verificationPeriod = 3; // Must match program's VERIFICATION_PERIOD
  //await new Promise(resolve => setTimeout(resolve, (verificationPeriod + 1) * 1000));
  await new Promise(resolve => setTimeout(resolve, 4 * 1000));
  
  await program.methods.finalizeVerification(caseId)
    .accounts({ patientCase: patientCasePda })
    .rpc();

  // Verify approval
  const updatedCase = await program.account.patientCase.fetch(patientCasePda);
  assert.deepEqual(updatedCase.status, { approved: {} });
  console.log("casestatus:", updatedCase.status);
});

it('Should create an escrow for approved case', async () => {
   // Verify case is approved before proceeding
    const caseAccount = await program.account.patientCase.fetch(patientCasePda);
   
    assert.isTrue(caseAccount.status.hasOwnProperty('approved'));
    
    // Generate escrow PDA using actual case ID
    const [escrowPda] = anchor.web3.PublicKey.findProgramAddressSync(

      [Buffer.from("escrow"), caseId.toBuffer('le', 8)],
      program.programId
    );
    console.log("escrowPda:", escrowPda.toString());

    await program.methods.createEscrow(caseId)
      .accounts({
        patientCase: patientCasePda,
        escrow: escrowPda,
        trustedEntity: trustedEntity.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([trustedEntity])
      .rpc();
      //console.log("Ayman'sPda thishastowork:", escrowPda.toString());

    // Verify escrow creation
    const escrowAccount = await program.account.escrowPda.fetch(escrowPda);
    assert.equal(escrowAccount.caseId.toString(), caseId.toString());
    assert.equal(escrowAccount.amount.toNumber(), 0);
    //assert.equal(escrowAccount.amount.toNumber(), 0, "Escrow should start with 0 balance");
    //console.log("escrowPda:", escrowPda);
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

    await program.methods.donate(donationAmount)
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
  
    const isCompliant = await program.methods.checkCompliance()
      .accounts({
        donor: donor.publicKey
      })
      .view();

    assert.equal(isCompliant, true);
  });
});