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
  let verifierRegistryPda: anchor.web3.PublicKey;
  let verificationPda: anchor.web3.PublicKey;
  let verifierPda: anchor.web3.PublicKey;  // Also needed for whitelist test
  let donationPda: anchor.web3.PublicKey;
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

  // Generate PDA for patient case 
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
    [verifierPda] = anchor.web3.PublicKey.findProgramAddressSync(
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
    [verifierRegistryPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("verifier"), verifier.publicKey.toBytes()],
        program.programId
    );

    // 2. Generate verification PDA (matches program seeds)
    [verificationPda] = anchor.web3.PublicKey.findProgramAddressSync(
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
 
  const verificationPeriod = 3; // Must match program's VERIFICATION_PERIOD
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
    globalEscrowPda = escrowPda;
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
      console.log("Escrow PDA:", escrowPda.toString());
    // Verify escrow creation
    const escrowAccount = await program.account.escrowPda.fetch(escrowPda);
    assert.equal(escrowAccount.caseId.toString(), caseId.toString());
    assert.equal(escrowAccount.amount.toNumber(), 0);
    //assert.equal(escrowAccount.amount.toNumber(), 0, "Escrow should start with 0 balance");
    console.log("createdescrowPdaforapprovedcase:", escrowPda);
    assert.ok(escrowAccount);
    globalEscrowPda = escrowPda;
   
});

let globalEscrowPda: anchor.web3.PublicKey;

it('Should process donations to escrow', async () => {
  const donationAmount = new BN(anchor.web3.LAMPORTS_PER_SOL); // 1 SOL

  // Fetch escrow and case ID
  const escrowAccount = await program.account.escrowPda.fetch(globalEscrowPda);
  const caseId = escrowAccount.caseId;

  // Generate donation PDA
  [donationPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
          Buffer.from("donation"),
          donor.publicKey.toBuffer(),
          caseId.toBuffer('le', 8)
      ],
      program.programId
  );

  // Process donation
  await program.methods.donate(donationAmount)
      .accounts({
          escrow: globalEscrowPda,
          donation: donationPda,
          donor: donor.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([donor])
      .rpc();

  // Verify the donation
  const updatedEscrow = await program.account.escrowPda.fetch(globalEscrowPda);
  assert.equal(updatedEscrow.amount.toNumber(), donationAmount.toNumber());
  console.log("donationtoescrow:", donationAmount);
  console.log("Patient CasePda:", patientCasePda.toString());
});


it('Should generate a funding report', async () => {
  // Get the escrow PDA using the case ID
  const caseAccount = await program.account.patientCase.fetch(patientCasePda);
  //const caseId = new BN(caseAccount.caseId.toString());
  const escrowPda = globalEscrowPda;
  console.log("escrow CasePda1:", escrowPda.toString());
  console.log("Patient CasePda1:", patientCasePda.toString());
  console.log("Donation Sucessfull:", escrowPda.toString());
});


/*
it('Should prevent duplicate votes from same verifier', async () => {
  try {
      await program.methods.verifyCase(true)
          .accounts({
              patientCase: patientCasePda,
              verifierRegistry: verifierRegistryPda,
              verifier: verifier.publicKey,
              verificationPda: verificationPda,
              systemProgram: SystemProgram.programId
          })
          .signers([verifier])
          .rpc();
          
      assert.fail("Should have thrown duplicate vote error");
  } catch (err) {
      // Check for both possible error representations
      assert.match(
          err.message,
          /DuplicateVote|0x1770/i,  
          "Expected DuplicateVote error"
      );
  }
});

it('Should reject zero amount donations', async () => {
  try {
      await program.methods.donate(new BN(0))
          .accounts({
              escrow: globalEscrowPda,
              donation: donationPda,
              donor: donor.publicKey,
              systemProgram: SystemProgram.programId
          })
          .signers([donor])
          .rpc();
          
      assert.fail("Should have thrown invalid amount error");
  } catch (err) {
      assert.include(err.message, "InvalidDonationAmount");
  }
});

it('Should prevent non-admin from whitelisting', async () => {
  const fakeAdmin = anchor.web3.Keypair.generate();
  
  try {
      await program.methods.whitelistVerifier(verifier.publicKey, "Nurse")
          .accounts({
              verifierRegistry: verifierPda,
              admin: fakeAdmin.publicKey,
              systemProgram: SystemProgram.programId
          })
          .signers([fakeAdmin])
          .rpc();
          
      assert.fail("Should have thrown unauthorized error");
  } catch (err) {
      assert.include(err.message, "Unauthorized");
  }
});

it('Should recognize a donor', async () => {
  const [recognitionPda] = await anchor.web3.PublicKey.findProgramAddressSync(
      [
          Buffer.from("recognition"),
          caseId.toBuffer('le', 8),
          donor.publicKey.toBuffer()
      ],
      program.programId
  );

  await program.methods.recognizeDonor("Platinum Supporter")
      .accounts({
          recognition: recognitionPda,
          admin: admin.publicKey,
          donor: donor.publicKey,
          caseCounter: caseCounterPda,
          systemProgram: SystemProgram.programId
      })
      .signers([admin])
      .rpc();

  const recognition = await program.account.donorRecognition.fetch(recognitionPda);
  assert.equal(recognition.recognitionType, "Platinum Supporter");
});

it('Should track case status correctly', async () => {
  const txSig = await program.methods.trackStatus()
      .accounts({
          patientCase: patientCasePda
      })
      .rpc();

  const tx = await connection.getTransaction(txSig);
  const logs = tx.meta.logMessages.join('\n');
  assert.include(logs, "Case Status");
  assert.include(logs, "Approved");
});

it('Should generate a funding report', async () => {
  
  // Fetch escrow account data
  const escrowAccount = await program.account.escrowPda.fetch(escrowPda);
  // Generate the report
  const transactionSignature = await program.methods.generateReport()
      .accounts({
          patientCase: patientCasePda,
          escrow: escrowPda
      })
      .rpc();
      console.log("transaction signature:", transactionSignature.toString());

  // Verify logs
  const transaction = await connection.getParsedTransaction(transactionSignature, {
      commitment: 'confirmed',
  });

  const logs = transaction.meta.logMessages.join('\n');
  assert.include(logs, "Case Report - ID:");
  assert.include(logs, "Approved");
  assert.include(logs, escrowAccount.amount.toString()); 
});

  it('Should check donor compliance', async () => {
  
    const isCompliant = await program.methods.checkCompliance()
      .accounts({
        donor: donor.publicKey
      })
      .view();

    assert.equal(isCompliant, true);
  });
  */
});
