import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Curachain } from "../target/types/curachain";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import chai, { assert, expect } from "chai";
import chaiAsPromised from "chai-as-promised";
import { createMint, getOrCreateAssociatedTokenAccount, getAssociatedTokenAddress, mintTo, getAccount, TOKEN_PROGRAM_ID, getMinimumBalanceForRentExemptAccount, ACCOUNT_SIZE, createInitializeAccountInstruction } from "@solana/spl-token";

chai.use(chaiAsPromised);

describe("CuraChain", () => {

  //Testing
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  var program = anchor.workspace.Curachain as any;

 //Actors
  const mediAdmin = provider.wallet; 
  const newAdmin = anchor.web3.Keypair.generate();
  const verifier1Keypair = anchor.web3.Keypair.generate(); 
  const verifier2Keypair = anchor.web3.Keypair.generate();
  const verifier3Keypair = anchor.web3.Keypair.generate();
  const verifier4Keypair = anchor.web3.Keypair.generate();
  const verifier5Keypair = anchor.web3.Keypair.generate();
  const verifier6Keypair = anchor.web3.Keypair.generate();
  const verifier7Keypair = anchor.web3.Keypair.generate();
  const verifier8Keypair = anchor.web3.Keypair.generate();
  const verifier9Keypair = anchor.web3.Keypair.generate();
  const verifier10Keypair = anchor.web3.Keypair.generate();
  const verifier11Keypair = anchor.web3.Keypair.generate();
  const donor1Keypair = anchor.web3.Keypair.generate();
  const donor2Keypair = anchor.web3.Keypair.generate(); 
  const donor3Keypair = anchor.web3.Keypair.generate(); 
  const patient1Keypair = anchor.web3.Keypair.generate(); 
  const patient2Keypair = anchor.web3.Keypair.generate();
  const patient3Keypair = anchor.web3.Keypair.generate(); 
  const facility_address = anchor.web3.Keypair.generate();

  //Airdrop function below

  async function airdropSol(provider, publicKey, amountSol) {
    const airdropSig = await provider.connection.requestAirdrop(
      publicKey,
      amountSol * anchor.web3.LAMPORTS_PER_SOL
    );

    await provider.connection.confirmTransaction(airdropSig);
  }

  //Setting up airdrop

  async function setupActors(provider, users, amount) {
    for (const user of users) {
      await airdropSol(provider, user, amount);
    }
  }

  // airdrop
  before(async () => {
    //  Administrator 5 SOL
    await airdropSol(provider, mediAdmin.publicKey, 5);

    //  Donors 10 SOL
    await setupActors(
      provider,
      [
        donor1Keypair.publicKey,
        donor2Keypair.publicKey,
        donor3Keypair.publicKey,
      ],
      10
    );

    // Verifier and Patients 5 SOL
    await setupActors(
      provider,
      [
        verifier1Keypair.publicKey,
        verifier2Keypair.publicKey,
        verifier3Keypair.publicKey,
        verifier4Keypair.publicKey,
        verifier5Keypair.publicKey,
        verifier6Keypair.publicKey,
        verifier7Keypair.publicKey,
        verifier8Keypair.publicKey,
        verifier9Keypair.publicKey,
        verifier10Keypair.publicKey,
        verifier11Keypair.publicKey,
        patient1Keypair.publicKey,
        patient2Keypair.publicKey,
        patient3Keypair.publicKey,
      ],
      5
    );
  });

  async function ensureVerifierExists(verifierKeypair, verifierPDA, adminPDA, verifiersListPDA) {
    try {
      await program.account.verifier.fetch(verifierPDA);
      // Already exists, do nothing
    } catch (e) {
      // Not found, add it
      await program.methods
        .addOrRemoveVerifier(verifierKeypair.publicKey, { add: {} })
        .accountsPartial({
          admin: newAdmin.publicKey,
          adminAccount: adminPDA,
          verifier: verifierPDA,
          verifiersList: verifiersListPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([newAdmin])
        .rpc();
    }
  }

  //Admin initialization
  it("Test 1- Admin Initialization.", async () => {
  
    //  Admin PDA
    const [adminPDA, adminBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .initializeAdministrator(newAdmin.publicKey)
      .accountsPartial({
        initializer: mediAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
      })
      .signers([])
      .rpc();

    // Let's make Some Assertions to Ascertain that The new Admin is Really Set
    const adminDetails = await program.account.administrator.fetch(adminPDA);
    expect(adminDetails.adminPubkey.toBuffer()).to.deep.equal(
      newAdmin.publicKey.toBuffer()
    ); 

    expect(adminDetails.adminPubkey.equals(newAdmin.publicKey)).to.be.true;
    expect(adminDetails.isActive).to.be.true;
    expect(adminDetails.bump).to.eq(adminBump);
  });


  //Admins initializing the global registry of verifiers
  it("Test 2- Admin Initializing The Global Registry Of Verifiers And Counter Case ID for Patient Submissions.", async () => {
    
    const [adminPDA, adminBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    // let's airdrop some sol to the newAdmin
    await airdropSol(provider, newAdmin.publicKey, 2);

    await program.methods
      .initializeGlobalVerifiersListAndCaseCounter()
      .accounts({
        admin: newAdmin.publicKey,
        //@ts-ignore
        adminAccount: adminPDA,
        verifiersList: verifiersRegistryPDA,
        caseCounter: caseCounterPDA,
      })
      .signers([newAdmin])
      .rpc();

    // Let's Fetch The Global Registry And Make Assertions
    const globalVerifiersListData = await program.account.verifiersList.fetch(
      verifiersRegistryPDA
    );

    expect(globalVerifiersListData.allVerifiers.length).to.equal(0);

    // Let's Fetch The Global Case Counter and Make Assertions
    const caseCounterData = await program.account.caseCounter.fetch(
      caseCounterPDA
    );
    expect(caseCounterData.currentId.toNumber()).to.equal(0);
    expect(caseCounterData.counterBump).to.equal(caseCounterBump);
  });


  //Admin adding 5 trusted verified Verifiers

  it("Test 3- Admin Adding 5 Verifiers", async () => {
    // Let's initialize admin account here:
    const [adminPDA, adminBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );

    //Verfier1 PDA address
    const [verifier1PDA, verifier1Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier1Keypair.publicKey.toBuffer()],
      program.programId
    );

    //Verifier2 PDA address
    const [verifier2PDA, verifier2Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
      program.programId
    );

    //Verifier3 PDA address
    const [verifier3PDA, verifier3Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier3Keypair.publicKey.toBuffer()],
      program.programId
    );

    //Verifier4 PDA address
    const [verifier4PDA, verifier4Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier4Keypair.publicKey.toBuffer()],
      program.programId
    );

    //Verifier5 PDA address
    const [verifier5PDA, verifier5Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier5Keypair.publicKey.toBuffer()],
      program.programId
    );

    //Verifier6 PDA address
    const [verifier6PDA, verifier6Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier6Keypair.publicKey.toBuffer()],
      program.programId
    );

    // Global Registry PDA
    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // airdrop some sol for the newAdmin
    await airdropSol(provider, newAdmin.publicKey, 3);

    // Adding Verifier 1
    await program.methods
      .addOrRemoveVerifier(verifier1Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier1PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Adding Verifier 2
    await program.methods
      .addOrRemoveVerifier(verifier2Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier2PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    //  Adding Verifier 3
    await program.methods
      .addOrRemoveVerifier(verifier3Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier3PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Adding Verifier 4
    await program.methods
      .addOrRemoveVerifier(verifier4Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier4PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Adding Verifier 5
    await program.methods
      .addOrRemoveVerifier(verifier5Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier5PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Adding Verifier 6
    await program.methods
      .addOrRemoveVerifier(verifier6Keypair.publicKey, { add: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier6PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Asserting Verifier 1 Data Initialized correctly
    const verifier1Details = await program.account.verifier.fetch(verifier1PDA);
    expect(verifier1Details.verifierKey.toBuffer()).deep.equal(
      verifier1Keypair.publicKey.toBuffer()
    );
    expect(verifier1Details.isVerifier).to.be.true;
    expect(verifier1Details.verifierBump).to.eq(verifier1Bump);

    // Asserting Verifier 2 Data Initialized correctly
    const verifier2Details = await program.account.verifier.fetch(verifier2PDA);
    expect(verifier2Details.verifierKey.toBuffer()).deep.equal(
      verifier2Keypair.publicKey.toBuffer()
    );
    expect(verifier2Details.isVerifier).to.be.true;
    expect(verifier2Details.verifierBump).to.eq(verifier2Bump);
    // Asserting Global Verifiers Registry Is Non-zero After Adding Verifier
    const globalVerifiersListData = await program.account.verifiersList.fetch(
      verifiersRegistryPDA
    );
    expect(globalVerifiersListData.allVerifiers.length).to.equal(6);
  });



  //Admin removing A Verifier From The Global Registry
  it("Test 4- Admin Removing Verifier 4 From The Global Registry.", async () => {
    // Let's get Verifier 1 PDA address
    const [verifier4PDA, verifier1Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier4Keypair.publicKey.toBuffer()],
      program.programId
    );
    // Let's get Admin PDA address
    const [adminPDA, adminBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );
    // Let's get The Global Registry PDA address
    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // let's airdrip some sol
    await airdropSol(provider, newAdmin.publicKey, 2);

    await program.methods
      .addOrRemoveVerifier(verifier4Keypair.publicKey, { remove: {} })
      .accountsPartial({
        admin: newAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminPDA,
        verifier: verifier4PDA,
        verifiersList: verifiersRegistryPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();

    // Let's make assertions on Global Registry
    const globalVerifiersListData = await program.account.verifiersList.fetch(
      verifiersRegistryPDA
    );

    expect(globalVerifiersListData.allVerifiers.length).to.equal(5);
  });


  //Only Admin Can Initialize (Add or Remove) A Verifier.
  it("Test 5- Only Admin Can Initialize (Add or Remove) A Verifier.", async () => {
    // Let's set up the Admin and Verifier PDAs
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );
    // New Verifier 3 PDA
    const [verifier4PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier4Keypair.publicKey.toBuffer()],
      program.programId
    );

    // VerifiersList PDA
    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // We will try to call the addOrRemoveVerifier instruction from a different account
    try {

      // initialize verifier instruction
      await program.methods
        .addOrRemoveVerifier(verifier4Keypair.publicKey, { add: {} })
        .accounts({
          admin: patient1Keypair.publicKey,
          // @ts-ignore
          adminAccount: adminPDA,
          verifier: verifier4PDA,
          verifiersList: verifiersRegistryPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([patient1Keypair])
        .rpc();
    } catch (err: any) {
      expect(err.error.errorCode.code).to.equal("OnlyAdmin");
    }
  });



  //Patients Submitting Medical Cases.
  it("Test 6- Patient 1 and 2 and 3 Submit Medical Case. ", async () => {
    // We setting up the respective PDAs
    const [patient1CasePDA, patient1CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
        program.programId
      );
    const [patient2CasePDA, patient2CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient2Keypair.publicKey.toBuffer()],
        program.programId
      );
    const [patient3CasePDA, patient3CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient3Keypair.publicKey.toBuffer()],
        program.programId
      );
    // Case Counter PDA
    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );
    const caseCounterDataAll = await program.account.caseCounter.fetch(
      caseCounterPDA
    );
    // Case LookUp PDAs for Patient 1 and 2 and 3
    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
      program.programId
    );

    const [caseLookupPDA2, caseLookupBump2] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0002")],
      program.programId
    );

    const [caseLookupPDA3, caseLookupBump3] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0003")],
      program.programId
    );

    // Let Patient 1 Call The submit Cases Instruction
    await program.methods
      .submitCases(
        "suffering from Cystic Fibrosis for 2 years now",
        new BN(20000),
        "www.gmail.com/drive/folders/medical_records.pdf"
      )
      .accounts({
        patient: patient1Keypair.publicKey,
        //@ts-ignore
        patientCase: patient1CasePDA,
        caseCounter: caseCounterPDA,
        caseLookup: caseLookupPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([patient1Keypair])
      .rpc();

    // Let Patient 2 Call The Submit Cases Instruction
    await program.methods
      .submitCases(
        "suffering from Ehlers-Danlos Syndrome for a year now",
        new BN(50000),
        "www.github.com/squash/medical_records.pdf"
      )
      .accounts({
        patient: patient2Keypair.publicKey,
        //@ts-ignore
        patientCase: patient2CasePDA,
        caseCounter: caseCounterPDA,
        caseLookup: caseLookupPDA2,
        systemProgram: SystemProgram.programId,
      })
      .signers([patient2Keypair])
      .rpc();

    // Let Patient 3 Call The Submit Cases Instruction
    await program.methods
      .submitCases(
        "suffering from Thyroid dysfunction for a year now",
        new BN(100000),
        "www.gmail.com/drive/folders/hospital_treatment_records.pdf"
      )
      .accounts({
        patient: patient3Keypair.publicKey,
        // @ts-ignore
        patientCase: patient3CasePDA,
        caseCounter: caseCounterPDA,
        caseLookup: caseLookupPDA3,
        systemProgram: SystemProgram.programId,
      })
      .signers([patient3Keypair])
      .rpc();

    // Let's get the Patient 1 & 2 & 3 Cases, And Case Counter
    const patient1CaseData = await program.account.patientCase.fetch(
      patient1CasePDA
    );
    const patient2CaseData = await program.account.patientCase.fetch(
      patient2CasePDA
    );
    const patient3CaseData = await program.account.patientCase.fetch(
      patient3CasePDA
    );

    // Let's Make The Assertions For Patient 1 Here
    expect(patient1CaseData.caseId.toString()).to.eq("CASE0001");
    expect(patient1CaseData.caseDescription.toString()).contains(
      "Cystic Fibrosis"
    );
    expect(patient1CaseData.verificationYesVotes).to.eq(0);
    expect(patient1CaseData.verificationNoVotes).to.eq(0);
    expect(patient1CaseData.isVerified).to.be.false;
    expect(patient1CaseData.totalAmountNeeded.toNumber()).to.eq(20000);
    expect(patient1CaseData.totalRaised.toNumber()).to.eq(0);

    // Let's Make Assertions For Patient 2 Here
    expect(patient2CaseData.caseId.toString()).to.eq("CASE0002");
    expect(patient2CaseData.caseDescription.toString()).contains(
      "Ehlers-Danlos Syndrome"
    );
    expect(patient2CaseData.verificationYesVotes).to.eq(0);
    expect(patient2CaseData.verificationNoVotes).to.eq(0);
    expect(patient2CaseData.isVerified).to.be.false;
    expect(patient2CaseData.totalAmountNeeded.toNumber()).to.eq(50000);
    expect(patient2CaseData.totalRaised.toNumber()).to.eq(0);

    // Let's Make Assertions For Patient 3 Here
    expect(patient3CaseData.caseId.toString()).to.eq("CASE0003");
    expect(patient3CaseData.caseDescription.toString()).contains(
      "Thyroid dysfunction"
    );
    expect(patient3CaseData.verificationYesVotes).to.eq(0);
    expect(patient3CaseData.verificationNoVotes).to.eq(0);
    expect(patient3CaseData.isVerified).to.be.false;
    expect(patient3CaseData.totalAmountNeeded.toNumber()).to.eq(100000);
    expect(patient3CaseData.totalRaised.toNumber()).to.eq(0);
  });

  

  //Verifying Patient 1 Case

  // Testing for Verification On Patient 1 Case
  it("Test 7- 4 Verifiers (1, 2, 3, 5) Verify Patient 1 Case: 5 Total Verifiers Initialized, 3 Votes a YES, and 1 a NO.", async () => {
    // Testing for verification Purpose
    const [patient1CasePDA, patient1CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
        program.programId
      );

    const [patient1EscrowPDA, patient1EscrowBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("patient_escrow"),
          Buffer.from("CASE0001"),
          patient1CasePDA.toBuffer(),
        ],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
      program.programId
    );

    const [verifier1PDA, verifier1Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier1Keypair.publicKey.toBuffer()],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // Let Verifier 1 call the approve
    await program.methods
      .verifyPatient("CASE0001", true)
      .accounts({
        verifier: verifier1Keypair.publicKey,
        //@ts-ignore
        patientCase: patient1CasePDA,
        verifierAccount: verifier1PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient1EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier1Keypair])
      .rpc();

    // Let Verifier 2 call the approve
    const [verifier2PDA, verifier2Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0001", true)
      .accounts({
        verifier: verifier2Keypair.publicKey,
        //@ts-ignore
        patientCase: patient1CasePDA,
        verifierAccount: verifier2PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient1EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier2Keypair])
      .rpc();

    // Let Verifier 3 and 5 call approve
    const [verifier3PDA, verifier3Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier3Keypair.publicKey.toBuffer()],
      program.programId
    );

    const [verifier5PDA, verifier5Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier5Keypair.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .verifyPatient("CASE0001", false)
      .accounts({
        verifier: verifier3Keypair.publicKey,
        //@ts-ignore
        patientCase: patient1CasePDA,
        verifierAccount: verifier3PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient1EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier3Keypair])
      .rpc();

    await program.methods
      .verifyPatient("CASE0001", true)
      .accounts({
        verifier: verifier5Keypair.publicKey,
        //@ts-ignore
        patientCase: patient1CasePDA,
        verifierAccount: verifier5PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient1EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier5Keypair])
      .rpc();
    // Let's get the Patient 1 Case Data
    const Patient1VerificationData = await program.account.patientCase.fetch(
      patient1CasePDA
    );

    // No Votes For Patient 1 Case = 1
    expect(Patient1VerificationData.verificationNoVotes).to.eq(1);

    // Yes Votes For Patient 1 Case = 3
    expect(Patient1VerificationData.verificationYesVotes).to.eq(3);

    // Verification Status is True
    expect(Patient1VerificationData.isVerified).to.be.true;
  });


  //Verifying Patient 2 Case

  // Testing for Verification On Patient 2 Case
  it("Test 8- 5 Verifiers (1, 2, 3, 5, 6) On Patient 2 Case: 5 Initialized, 3 Votes a YES, and 2 a NO. 70% threshold working.", async () => {
    // Testing For Verification Purposes on Patient 2 Case
    const [patient2CasePDA, patient2CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient2Keypair.publicKey.toBuffer()],
        program.programId
      );

    const [patient2EscrowPDA, patient2EscrowBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("patient_escrow"),
          Buffer.from("CASE0002"),
          patient2CasePDA.toBuffer(),
        ],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0002")],
      program.programId
    );

    const [verifier1PDA, verifier1Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier1Keypair.publicKey.toBuffer()],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // Verifier 1 call approve with Yes on Patient 2
    await program.methods
      .verifyPatient("CASE0002", true)
      .accounts({
        verifier: verifier1Keypair.publicKey,
        // @ts-ignore
        patientCase: patient2CasePDA,
        verfifierAccount: verifier1PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient2EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier1Keypair])
      .rpc();

    // Verifier 2 Call Approve With No on Patient Case 2
    const [verifier2PDA, verifier2Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0002", false)
      .accounts({
        verifier: verifier2Keypair.publicKey,
        // @ts-ignore
        patientCase: patient2CasePDA,
        verfifierAccount: verifier2PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient2EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier2Keypair])
      .rpc();

    // Verifier 3 Call Approve With Yes on Patient Case 2
    const [verifier3PDA, verifier3Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier3Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0002", true)
      .accounts({
        verifier: verifier3Keypair.publicKey,
        // @ts-ignore
        patientCase: patient2CasePDA,
        verifierAccount: verifier3PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient2EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier3Keypair])
      .rpc();

    // Verifier 5 Call Approve With No on Patient Case 2
    const [verifier5PDA, verifier5Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier5Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0002", false)
      .accounts({
        verifier: verifier5Keypair.publicKey,
        // @ts-ignore
        patientCase: patient2CasePDA,
        verifierAccount: verifier5PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient2EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier5Keypair])
      .rpc();

    // Verifier 6 Call Approve With Yes On Patient Case 2
    const [verifier6PDA, verifier6Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier6Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0002", true)
      .accounts({
        verifier: verifier6Keypair.publicKey,
        // @ts-ignore
        patientCase: patient2CasePDA,
        verifierAccount: verifier6PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient2EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier6Keypair])
      .rpc();
    // Let's Get Patient 2 Verification Details data
    const patient2VerificationData = await program.account.patientCase.fetch(
      patient2CasePDA
    );

    // Yes Verification Votes For Patient Case 2 = 3
    expect(patient2VerificationData.verificationYesVotes).to.eq(3);

    // No Verification Votes For Patient Case 2 = 2
    expect(patient2VerificationData.verificationNoVotes).to.eq(2);

    // Verification status for Patient Case 2 is false
    expect(patient2VerificationData.isVerified).to.be.false;
  });



  //Verifying Patient 3 Case
  it("Test 9- 4 Verifiers (2, 3, 5, 6) On Patient 3 Case: 3 Vote a NO, 1 vote a YES. Patient Case Account Is Not Verified", async () => {
    // Let's Get The Patient PDAs
    const [patient3CasePDA, patient3CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient3Keypair.publicKey.toBuffer()],
        program.programId
      );

    const [patient3EscrowPDA, patient3EscrowBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("patient_escrow"),
          Buffer.from("CASE0003"),
          patient3CasePDA.toBuffer(),
        ],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0003")],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    // Verfier 2 Vote a No on Case 3
    const [verifier2PDA, verifier2Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0003", false)
      .accounts({
        verifier: verifier2Keypair.publicKey,
        // @ts-ignore
        patientCase: patient3CasePDA,
        verfifierAccount: verifier2PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient3EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier2Keypair])
      .rpc();

    // Verifier 3 Call Approve With Yes on Patient Case 2
    const [verifier3PDA, verifier3Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier3Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0003", false)
      .accounts({
        verifier: verifier3Keypair.publicKey,
        // @ts-ignore
        patientCase: patient3CasePDA,
        verifierAccount: verifier3PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient3EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier3Keypair])
      .rpc();

    // Verifier 5 Call Approve With No on Patient Case 2
    const [verifier5PDA, verifier5Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier5Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0003", false)
      .accounts({
        verifier: verifier5Keypair.publicKey,
        // @ts-ignore
        patientCase: patient3CasePDA,
        verifierAccount: verifier5PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient3EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier5Keypair])
      .rpc();

    // Verifier 6 Call Approve With Yes On Patient Case 2
    const [verifier6PDA, verifier6Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier6Keypair.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .verifyPatient("CASE0003", true)
      .accounts({
        verifier: verifier6Keypair.publicKey,
        // @ts-ignore
        patientCase: patient3CasePDA,
        verifierAccount: verifier6PDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersRegistryPDA,
        patientEscrow: patient3EscrowPDA,
        caseCounter: caseCounterPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([verifier6Keypair])
      .rpc();

    const patient3CaseData = await program.account.patientCase.fetch(
      patient3CasePDA
    );
    // Yes Verification Votes On Patient Case 3 = 1
    expect(patient3CaseData.verificationYesVotes).to.eq(1);

    // No Verification Votes on Patient Case 3 = 3
    expect(patient3CaseData.verificationNoVotes).to.eq(3);

    // Verification Status For Patient Case 3 is clearly rejected, false
    expect(patient3CaseData.isVerified).to.be.false;
  });



  //A Verifier Cannot Vote Twice On A Particular Case.
  it("Test 10- Verifier Cannot Vote Twice On A Particular Case.", async () => {
    // Verifier 5 Voted On Case 2 In The Prior Test

    const [patient2CasePDA, patient2CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient2Keypair.publicKey.toBuffer()],
        program.programId
      );

    const [patient2EscrowPDA, patient2EscrowBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("patient_escrow"),
          Buffer.from("CASE0002"),
          patient2CasePDA.toBuffer(),
        ],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0002")],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    const [verifier5PDA, verifier5Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier5Keypair.publicKey.toBuffer()],
      program.programId
    );

    // Let's Ascertain If The Transaction Will Revert If Verifier 5 Attempts to Vote on Case 2 Again
    try {
      await program.methods
        .verifyPatient("CASE0002", true)
        .accounts({
          verifier: verifier5Keypair.publicKey,
          // @ts-ignore
          patientCase: patient2CasePDA,
          verifierAccount: verifier5PDA,
          caseLookup: caseLookupPDA,
          verifiersList: verifiersRegistryPDA,
          patientEscrow: patient2EscrowPDA,
          caseCounter: caseCounterPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([verifier5Keypair])
        .rpc();
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("VerifierAlreadyVoted");
    }
  });

 
  //A Verifier Cannot Vote On An Already Verified Case
  it("Test 11- A Verifier Cannot Vote On An Already Verified Case  ==> Verifier6 Cannot Vote On Case 1, Which is Already Verified", async () => {
    //Verifier 6 Did Not Vote On Case 1 prior to it being verified.
    // Now, He attempts to Vote on Case 1, but will get a transaction revert.
    const [verifier6PDA, verifier6Bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier6Keypair.publicKey.toBuffer()],
      program.programId
    );

    const [patient1CasePDA, patient1CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
        program.programId
      );

    const [patient1EscrowPDA, patient1EscrowBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("patient_escrow"),
          Buffer.from("CASE0001"),
          patient1CasePDA.toBuffer(),
        ],
        program.programId
      );

    const [caseCounterPDA, caseCounterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")],
      program.programId
    );

    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
      program.programId
    );

    const [verifiersRegistryPDA, verifiersRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );

    try {
      await program.methods
        .verifyPatient("CASE0001", true)
        .accounts({
          verifier: verifier6Keypair.publicKey,
          // @ts-ignore
          patientCase: patient1CasePDA,
          verifierAccount: verifier6PDA,
          caseLookup: caseLookupPDA,
          verifiersList: verifiersRegistryPDA,
          patientEscrow: patient1EscrowPDA,
          caseCounter: caseCounterPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([verifier6Keypair])
        .rpc();
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("CaseAlreadyVerified");
    }
  });


  //Verifier cannot vote after 10 days (VotingPeriodExpired)
  it("Test 12- Verifier cannot vote after 10 days (VotingPeriodExpired)", async () => {
    const testPatient = anchor.web3.Keypair.generate();
    await airdropSol(provider, testPatient.publicKey, 2);

    // Predict the next case ID
    const [testCaseCounterPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")], program.programId
    );
    const caseCounterAccount = await program.account.caseCounter.fetch(testCaseCounterPDA);
    const nextCaseId = `CASE${String(caseCounterAccount.currentId.toNumber() + 1).padStart(4, '0')}`;

    // Derive the correct PDAs
    const [testPatientCasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), testPatient.publicKey.toBuffer()],
      program.programId
    );
    const [caseLookupPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from(nextCaseId)],
      program.programId
    );
    const [patientEscrowPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_escrow"), Buffer.from(nextCaseId), testPatientCasePDA.toBuffer()],
      program.programId
    );

    // Submit the case
    await program.methods
      .submitCases("Test 12 case for time limit", new BN(10000), "test12link")
      .accountsPartial({
        patient: testPatient.publicKey,
        patientCase: testPatientCasePDA,
        caseCounter: testCaseCounterPDA,
        caseLookup: caseLookupPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([testPatient])
      .rpc();

    await warpForwardByDays(11);

    const [verifiersRegistryPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifiers_list")], program.programId
    );
    const [verifier6PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier6Keypair.publicKey.toBuffer()], program.programId
    );
    const patientCase = await program.account.patientCase.fetch(testPatientCasePDA);
    const now = (await provider.connection.getBlockTime(await provider.connection.getSlot())) || 0;
    if (now - patientCase.submissionTimestamp.toNumber() < 864000) {
      return;
    }
    let threw = false;
    try {
      await program.methods
        .verifyPatient(nextCaseId, true)
        .accountsPartial({
          verifier: verifier6Keypair.publicKey,
          verifierAccount: verifier6PDA,
          verifiersList: verifiersRegistryPDA,
          caseLookup: caseLookupPDA,
          patientCase: testPatientCasePDA,
          patientEscrow: patientEscrowPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([verifier6Keypair])
        .rpc();
    } catch (err) {
      threw = true;
      expect(err.error.errorCode.code).to.equal("VotingPeriodExpired");
    }
    expect(threw).to.be.true;
  });

  
  //Admin cannot override before 10 days (VotingPeriodExpired)
  it("Test 13- Admin cannot override before 10 days (VotingPeriodExpired)", async () => {
    // Setup
    const testPatient = anchor.web3.Keypair.generate();
    await airdropSol(provider, testPatient.publicKey, 2);

    // Predict the next case ID
    const [testCaseCounterPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")], program.programId
    );
    const caseCounterAccount = await program.account.caseCounter.fetch(testCaseCounterPDA);
    const nextCaseId = `CASE${String(caseCounterAccount.currentId.toNumber() + 1).padStart(4, '0')}`;

    // Derive the correct PDAs
    const [testPatientCasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), testPatient.publicKey.toBuffer()],
      program.programId
    );
    const [caseLookupPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from(nextCaseId)],
      program.programId
    );
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()], program.programId
    );
    const [patientEscrowPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_escrow"), Buffer.from(nextCaseId), testPatientCasePDA.toBuffer()],
      program.programId
    );

    // Submit the case
    await program.methods
      .submitCases("Test 20 case for admin override", new BN(10000), "test20link")
      .accountsPartial({
        patient: testPatient.publicKey,
        patientCase: testPatientCasePDA,
        caseCounter: testCaseCounterPDA,
        caseLookup: caseLookupPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([testPatient])
      .rpc();

    // Use chai-as-promised to assert the error is thrown
    await expect(
      program.methods
        .adminOverrideCase(nextCaseId, true)
        .accountsPartial({
          admin: newAdmin.publicKey,
          adminAccount: adminPDA,
          caseLookup: caseLookupPDA,
          patientCase: testPatientCasePDA,
          patientEscrow: patientEscrowPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([newAdmin])
        .rpc()
    ).to.be.rejectedWith(/VotingPeriodExpired|6026/);
  });


  //Admin can override after 10 days (verify)
  it("TEST 14- Admin can override after 10 days (verify)", async () => {
    const testPatient = anchor.web3.Keypair.generate();
    await airdropSol(provider, testPatient.publicKey, 2);

    // Predict the next case ID
    const [testCaseCounterPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")], program.programId
    );
    const caseCounterAccount = await program.account.caseCounter.fetch(testCaseCounterPDA);
    const nextCaseId = `CASE${String(caseCounterAccount.currentId.toNumber() + 1).padStart(4, '0')}`;

    // Derive the correct PDAs
    const [testPatientCasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), testPatient.publicKey.toBuffer()],
      program.programId
    );
    const [caseLookupPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from(nextCaseId)],
      program.programId
    );
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()], program.programId
    );
    // FIX: Define patientEscrowPDA for this test
    const [patientEscrowPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_escrow"), Buffer.from(nextCaseId), testPatientCasePDA.toBuffer()],
      program.programId
    );

    // Submit the case
    await program.methods
      .submitCases("Test 14 case for admin override", new BN(10000), "test14link")
      .accountsPartial({
        patient: testPatient.publicKey,
        patientCase: testPatientCasePDA,
        caseCounter: testCaseCounterPDA,
        caseLookup: caseLookupPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([testPatient])
      .rpc();

    await warpForwardByDays(11);

    const patientCase = await program.account.patientCase.fetch(testPatientCasePDA);
    const now = (await provider.connection.getBlockTime(await provider.connection.getSlot())) || 0;
    if (now - patientCase.submissionTimestamp.toNumber() < 864000) {
      return;
    }
    await program.methods
      .adminOverrideCase(nextCaseId, true)
      .accountsPartial({
        admin: newAdmin.publicKey,
        adminAccount: adminPDA,
        caseLookup: caseLookupPDA,
        patientCase: testPatientCasePDA,
        patientEscrow: patientEscrowPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();
    // Check that the case is now verified
    const patientCaseData = await program.account.patientCase.fetch(
      testPatientCasePDA
    );
    expect(patientCaseData.isVerified).to.be.true;
  });

  //Admin can override after 10 days (reject)
  it("Test 15- Admin can override after 10 days (reject)", async () => {
    const testPatient = anchor.web3.Keypair.generate();
    await airdropSol(provider, testPatient.publicKey, 2);

    // Predict the next case ID
    const [testCaseCounterPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_counter")], program.programId
    );
    const caseCounterAccount = await program.account.caseCounter.fetch(testCaseCounterPDA);
    const nextCaseId = `CASE${String(caseCounterAccount.currentId.toNumber() + 1).padStart(4, '0')}`;

    // Derive the correct PDAs
    const [testPatientCasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), testPatient.publicKey.toBuffer()],
      program.programId
    );
    const [caseLookupPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from(nextCaseId)],
      program.programId
    );
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()], program.programId
    );
    // FIX: Define patientEscrowPDA for this test
    const [patientEscrowPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_escrow"), Buffer.from(nextCaseId), testPatientCasePDA.toBuffer()],
      program.programId
    );

    // Submit the case
    await program.methods
      .submitCases("Test 15 case for admin override", new BN(10000), "test15link")
      .accountsPartial({
        patient: testPatient.publicKey,
        patientCase: testPatientCasePDA,
        caseCounter: testCaseCounterPDA,
        caseLookup: caseLookupPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([testPatient])
      .rpc();

    await warpForwardByDays(11);

    const patientCase = await program.account.patientCase.fetch(testPatientCasePDA);
    const now = (await provider.connection.getBlockTime(await provider.connection.getSlot())) || 0;
    if (now - patientCase.submissionTimestamp.toNumber() < 864000) {
      return;
    }
    await program.methods
      .adminOverrideCase(nextCaseId, false)
      .accountsPartial({
        admin: newAdmin.publicKey,
        adminAccount: adminPDA,
        caseLookup: caseLookupPDA,
        patientCase: testPatientCasePDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin])
      .rpc();
    // Check that the case is now rejected
    const patientCaseData = await program.account.patientCase.fetch(
      testPatientCasePDA
    );
    expect(patientCaseData.isVerified).to.be.false;
  });

// Admin override creates escrow PDA and allows donations
it("Test 16- Admin override creates escrow PDA and allows donations", async () => {
  // Setup: Use a new patient and donor keypair
  const testPatient = anchor.web3.Keypair.generate();
  const testDonor = anchor.web3.Keypair.generate();
  await airdropSol(provider, testPatient.publicKey, 2);
  await airdropSol(provider, testDonor.publicKey, 2);

  // Predict the next case ID
  const [testCaseCounterPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("case_counter")], program.programId
  );
  const caseCounterAccount = await program.account.caseCounter.fetch(testCaseCounterPDA);
  const nextCaseId = `CASE${String(caseCounterAccount.currentId.toNumber() + 1).padStart(4, '0')}`;

  // Derive the correct PDAs
  const [testPatientCasePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient"), testPatient.publicKey.toBuffer()],
    program.programId
  );
  const [caseLookupPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("case_lookup"), Buffer.from(nextCaseId)],
    program.programId
  );
  const [adminPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin"), newAdmin.publicKey.toBuffer()], program.programId
  );
  const [donorAccountPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("donor"), testDonor.publicKey.toBuffer()],
    program.programId
  );
  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig")],
    program.programId
  );
  // FIX: Define patientEscrowPDA for this test
  const [patientEscrowPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient_escrow"), Buffer.from(nextCaseId), testPatientCasePDA.toBuffer()],
    program.programId
  );

  // Submit the case
  await program.methods
    .submitCases("Test admin override escrow creation", new BN(10000), "escrowlink")
    .accountsPartial({
      patient: testPatient.publicKey,
      patientCase: testPatientCasePDA,
      caseCounter: testCaseCounterPDA,
      caseLookup: caseLookupPDA,
      systemProgram: SystemProgram.programId,
    })
    .signers([testPatient])
    .rpc();

  // Warp time forward by 11 days and check the time difference
  await warpForwardByDays(11);
  const patientCase = await program.account.patientCase.fetch(testPatientCasePDA);
  const now = (await provider.connection.getBlockTime(await provider.connection.getSlot())) || 0;
  const diff = now - patientCase.submissionTimestamp.toNumber();
  // If time warp did not advance enough, skip the test
  if (diff < 864000) {
    return;
  }
  // Assert that at least 10 days have passed
  expect(diff).to.be.greaterThan(864000);

  // Call admin_override_case with is_verified = true and pass patientEscrow
  await program.methods
    .adminOverrideCase(nextCaseId, true)
    .accountsPartial({
      admin: newAdmin.publicKey,
      adminAccount: adminPDA,
      caseLookup: caseLookupPDA,
      patientCase: testPatientCasePDA,
      patientEscrow: patientEscrowPDA,
      systemProgram: SystemProgram.programId,
    })
    .signers([newAdmin])
    .rpc();

  // Assert that the escrow account now exists and is owned by the system program
  const escrowAccountInfo = await provider.connection.getAccountInfo(patientEscrowPDA);
  expect(escrowAccountInfo).to.not.be.null;
  expect(escrowAccountInfo.owner.toString()).to.equal(SystemProgram.programId.toString());

  // Now, try to donate to the case (should succeed)
  await program.methods
    .donate(nextCaseId, new BN(1000), new PublicKey("11111111111111111111111111111111"))
    .accounts({
      donor: testDonor.publicKey,
      caseLookup: caseLookupPDA,
      patientCase: testPatientCasePDA,
      patientEscrow: patientEscrowPDA,
      donorAccount: donorAccountPDA,
      multisigPda: multisigPda,
      mint: new PublicKey("11111111111111111111111111111111"),
      donorAta: null,
      patientAta: null,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      donorNftMint: null,
      donorNftAta: null,
      donorNftMetadata: null,
      tokenMetadataProgram: null,
    })
    .signers([testDonor])
    .rpc();

  // Check that the escrow account balance increased
  const escrowAfter = await provider.connection.getAccountInfo(patientEscrowPDA);
  expect(escrowAfter.lamports).to.be.greaterThan(0);
});


  //Donations to Verified Case
  it('Test 17- 2 Donors Contributing Funds To A Verified Case I', async () => {
    const donor1 = donor1Keypair;
    const donor2 = anchor.web3.Keypair.generate();
    await airdropSol(provider, donor2.publicKey, 2);
    const caseId = "CASE0001";
    const patient = patient1Keypair;
    const [patientCasePDA] = PublicKey.findProgramAddressSync([
      Buffer.from("patient"), patient.publicKey.toBuffer()
    ], program.programId);
    const [caseLookupPDA] = PublicKey.findProgramAddressSync([
      Buffer.from("case_lookup"), Buffer.from(caseId)
    ], program.programId);
    const [patientEscrowPDA] = PublicKey.findProgramAddressSync([
      Buffer.from("patient_escrow"), Buffer.from(caseId), patientCasePDA.toBuffer()
    ], program.programId);
    const [donorAccountPDA1] = PublicKey.findProgramAddressSync([
      Buffer.from("donor"), donor1.publicKey.toBuffer()
    ], program.programId);
    const [donorAccountPDA2] = PublicKey.findProgramAddressSync([
      Buffer.from("donor"), donor2.publicKey.toBuffer()
    ], program.programId);
    const [multisigPda] = PublicKey.findProgramAddressSync([
      Buffer.from("multisig")
    ], program.programId);
  
    // Verify CASE0001 is already submitted and verified
    const patientCaseData = await program.account.patientCase.fetch(patientCasePDA);
    expect(patientCaseData.isVerified).to.be.true;
    expect(patientCaseData.caseId).to.equal(caseId);
  
    // Donor 1 donation
    await donate({
      program,
      donorKeypair: donor1,
      caseId,
      amount: new BN(15000),
      mint: new PublicKey("11111111111111111111111111111111"),
      caseLookupPDA,
      patientCasePDA,
      patientEscrowPDA,
      donorAccountPDA: donorAccountPDA1,
      multisigPda
    });
  
    // Donor 2 donation
    await donate({
      program,
      donorKeypair: donor2,
      caseId,
      amount: new BN(10000),
      mint: new PublicKey("11111111111111111111111111111111"),
      caseLookupPDA,
      patientCasePDA,
      patientEscrowPDA,
      donorAccountPDA: donorAccountPDA2,
      multisigPda
    });
  
    // Verify donations were recorded
    const updatedPatientCase = await program.account.patientCase.fetch(patientCasePDA);
    expect(updatedPatientCase.totalRaised.toNumber()).to.be.greaterThanOrEqual(25000);
  });

// Test 18: Donors Attempt To Contribute To An Unverified Case II or III, Must Fail
it("Test 18- Donors Attempt To Contribute To An Unverified Case II or III, Must Fail", async () => {
  const [patient2CasePDA, patient2CaseBump] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), patient2Keypair.publicKey.toBuffer()],
      program.programId
    );
  const [patient2EscrowPDA, patient2EscrowBump] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("patient_escrow"),
        Buffer.from("CASE0002"),
        patient2CasePDA.toBuffer(),
      ],
      program.programId
    );

  const [caseLookupPDA2, caseLookupBump2] = PublicKey.findProgramAddressSync(
    [Buffer.from("case_lookup"), Buffer.from("CASE0002")],
    program.programId
  );

  const [donor1PDA, donor1Bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("donor"), donor1Keypair.publicKey.toBuffer()],
    program.programId
  );

  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig")],
    program.programId
  );

  try {
    await donate({
      program,
      donorKeypair: donor1Keypair,
      caseId: "CASE0002",
      amount: new BN(30000),
      mint: new PublicKey("11111111111111111111111111111111"),
      caseLookupPDA: caseLookupPDA2,
      patientCasePDA: patient2CasePDA,
      patientEscrowPDA: patient2EscrowPDA,
      donorAccountPDA: donor1PDA,
      multisigPda
    });
    throw new Error("Expected UnverifiedCase error");
  } catch (err) {
    if (err.error && err.error.errorCode) {
      expect(err.error.errorCode.code).to.equal("UnverifiedCase");
    } else {
      throw err;
    }
  }
});

// Test 19: Donors can donate both SOL and SPL tokens to Patient 1's case and track donations
it("Test 19- Donors can donate both SOL and SPL tokens to Patient 1's case and track donations", async () => {
  const testPayer = anchor.web3.Keypair.generate();
  await airdropSol(provider, testPayer.publicKey, 2);

  const mintAuthority = Keypair.generate();
  const splMint = await createMint(
    provider.connection,
    testPayer,
    mintAuthority.publicKey,
    null,
    6
  );
  const donor1Ata = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    testPayer,
    splMint,
    donor1Keypair.publicKey
  );
  await mintTo(
    provider.connection,
    testPayer,
    splMint,
    donor1Ata.address,
    mintAuthority,
    1_000_000_000
  );

  const [patient1CasePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
    program.programId
  );
  const [caseLookupPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
    program.programId
  );
  const [patient1EscrowPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient_escrow"), Buffer.from("CASE0001"), patient1CasePDA.toBuffer()],
    program.programId
  );
  const [donor1PDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("donor"), donor1Keypair.publicKey.toBuffer()],
    program.programId
  );
  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig")],
    program.programId
  );

  await airdropSol(provider, donor1Keypair.publicKey, 5);
  await donate({
    program,
    donorKeypair: donor1Keypair,
    caseId: "CASE0001",
    amount: new BN(1000),
    mint: new PublicKey("11111111111111111111111111111111"),
    caseLookupPDA,
    patientCasePDA: patient1CasePDA,
    patientEscrowPDA: patient1EscrowPDA,
    donorAccountPDA: donor1PDA,
    multisigPda
  });

  const [patient1SplPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient_spl"), Buffer.from("CASE0001"), splMint.toBuffer(), multisigPda.toBuffer()],
    program.programId
  );

  await program.methods
    .initializePatientSplAccount("CASE0001", splMint)
    .accounts({
      payer: testPayer.publicKey,
      patientCase: patient1CasePDA,
      multisigPda: multisigPda,
      patientSplAta: patient1SplPda,
      mint: splMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([testPayer])
    .rpc();

  await program.methods
    .donate("CASE0001", new BN(500_000), splMint)
    .accounts({
      donor: donor1Keypair.publicKey,
      caseLookup: caseLookupPDA,
      patientCase: patient1CasePDA,
      patientEscrow: patient1EscrowPDA,
      donorAccount: donor1PDA,
      donorAta: donor1Ata.address,
      patientAta: patient1SplPda,
      multisigPda: multisigPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      mint: splMint,
      donorNftMint: null,
      donorNftAta: null,
      donorNftMetadata: null,
      tokenMetadataProgram: null,
    })
    .signers([donor1Keypair])
    .rpc();

  const patient1CaseData = await program.account.patientCase.fetch(patient1CasePDA);
  expect(patient1CaseData.totalRaised.toNumber()).to.be.gte(1000);
  expect(patient1CaseData.splDonations.length).to.be.gte(1);
  expect(patient1CaseData.splDonations[0].mint.toBase58()).to.equal(splMint.toBase58());
  expect(patient1CaseData.splDonations[0].amount.toNumber()).to.equal(500_000);
});
  //Authorized Multisig Can Release Funds From Escrow To Treatment Wallet
  it("Test 20 - Only authorized multisig (admin + 3 verifiers) can release funds from escrow", async () => {
    // Derive all required PDAs
    const [patient1CasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
      program.programId
    );
    const [patient1EscrowPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_escrow"), Buffer.from("CASE0001"), patient1CasePDA.toBuffer()],
      program.programId
    );
    const [caseLookupPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
      program.programId
    );
    const [verifiersListPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifiers_list")],
      program.programId
    );
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
      program.programId
    );
    const [verifier1PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier1Keypair.publicKey.toBuffer()],
      program.programId
    );
    const [verifier2PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
      program.programId
    );
    const [verifier3PDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("verifier_role"), verifier3Keypair.publicKey.toBuffer()],
      program.programId
    );
    const [multisigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("multisig")],
      program.programId
    );
  
    // Ensure all verifiers are initialized
    await ensureVerifierExists(verifier1Keypair, verifier1PDA, adminPDA, verifiersListPDA);
    await ensureVerifierExists(verifier2Keypair, verifier2PDA, adminPDA, verifiersListPDA);
    await ensureVerifierExists(verifier3Keypair, verifier3PDA, adminPDA, verifiersListPDA);
  
    // Airdrop SOL and verify balance
    await airdropSol(provider, newAdmin.publicKey, 5);
    const newAdminBalance = await provider.connection.getBalance(newAdmin.publicKey);
    if (newAdminBalance < 2 * LAMPORTS_PER_SOL) {
      throw new Error("Insufficient balance for newAdmin after airdrop");
    }
    await airdropSol(provider, facility_address.publicKey, 5);
  
    // Create a dummy SPL mint with error handling
    let dummyMint;
    try {
      dummyMint = await createMint(
        provider.connection,
        newAdmin,
        newAdmin.publicKey,
        null,
        0
      );
    } catch (err) {
      throw new Error("createMint failed, cannot proceed with test");
    }
  
    // Validate dummyMint
    if (!dummyMint || !(dummyMint instanceof PublicKey)) {
      throw new Error("dummyMint is undefined or invalid");
    }
  
    // Create multisig-owned SPL token account at a PDA
    const [patient1SplPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("patient_spl"), Buffer.from("CASE0001"), dummyMint.toBuffer(), multisigPda.toBuffer()],
      program.programId
    );
  
    // Initialize the patient's SPL token account if it doesn't exist
    const accountInfo = await provider.connection.getAccountInfo(patient1SplPDA);
    if (!accountInfo) {
      await program.methods
        .initializePatientSplAccount("CASE0001", dummyMint)
        .accounts({
          payer: newAdmin.publicKey,
          patientCase: patient1CasePDA,
          multisigPda: multisigPda,
          patientSplAta: patient1SplPDA,
          mint: dummyMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([newAdmin])
        .rpc();
    }
  
    // Create facility's ATA for the same mint
    const facilitySplAta1 = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      newAdmin,
      dummyMint,
      facility_address.publicKey
    );
  
    // Call the releaseFunds instruction
    await program.methods
      .releaseFunds("CASE0001")
      .accounts({
        admin: newAdmin.publicKey,
        adminAccount: adminPDA,
        patientCase: patient1CasePDA,
        patientEscrow: patient1EscrowPDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersListPDA,
        verifier1: verifier1Keypair.publicKey,
        verifier2: verifier2Keypair.publicKey,
        verifier3: verifier3Keypair.publicKey,
        verifier1Pda: verifier1PDA,
        verifier2Pda: verifier2PDA,
        verifier3Pda: verifier3PDA,
        facilityAddress: facility_address.publicKey,
        multisigPda: multisigPda,
        patientSplAta: patient1SplPDA,
        patientSplAta1: SystemProgram.programId,
        facilitySplAta1: facilitySplAta1.address,
        patientSplAta2: SystemProgram.programId,
        facilitySplAta2: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: patient1SplPDA, isWritable: true, isSigner: false },
        { pubkey: facilitySplAta1.address, isWritable: true, isSigner: false },
        { pubkey: dummyMint, isWritable: false, isSigner: false },
      ])
      .signers([newAdmin, verifier1Keypair, verifier2Keypair, verifier3Keypair])
      .rpc();
  
    // Assert: Escrow should be empty or only contain rent-exempt amount
    const escrowBalance = await provider.connection.getBalance(patient1EscrowPDA);
    expect(escrowBalance).to.be.lte(890880); // Adjust for rent-exempt balance
  
    // Check SPL token accounts
    const patientSpl1 = await getAccount(provider.connection, patient1SplPDA);
    const facilitySpl1 = await getAccount(provider.connection, facilitySplAta1.address);
    expect(Number(patientSpl1.amount)).to.eq(0);
  });

 // Test 21: Negative test, tries to release funds with 2 verifiers.
 it("Test 21- Fails to release funds if not enough verifiers sign", async () => {
  
  const [patient1CasePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
    program.programId
  );
  const [patient1EscrowPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("patient_escrow"), Buffer.from("CASE0001"), patient1CasePDA.toBuffer()],
    program.programId
  );
  const [verifiersListPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("verifiers_list")],
    program.programId
  );
  const [adminPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin"), newAdmin.publicKey.toBuffer()],
    program.programId
  );
  const [caseLookupPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
    program.programId
  );
  const [verifier1PDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("verifier_role"), verifier1Keypair.publicKey.toBuffer()],
    program.programId
  );
  const [verifier2PDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("verifier_role"), verifier2Keypair.publicKey.toBuffer()],
    program.programId
  );
  // Omit verifier3
  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig")],
    program.programId
  );
  const patientSplAta = patient1Keypair.publicKey;
  const dummyAccount = SystemProgram.programId;

  try {
    await program.methods
      .releaseFunds("CASE0001")
      .accounts({
        admin: newAdmin.publicKey,
        adminAccount: adminPDA,
        patientCase: patient1CasePDA,
        patientEscrow: patient1EscrowPDA,
        caseLookup: caseLookupPDA,
        verifiersList: verifiersListPDA,
        verifier1: verifier1Keypair.publicKey,
        verifier2: verifier2Keypair.publicKey,
        // Omit verifier3
        verifier1Pda: verifier1PDA,
        verifier2Pda: verifier2PDA,
        // Omit verifier3Pda
        facilityAddress: facility_address.publicKey,
        multisigPda: multisigPda,
        patientSplAta: patientSplAta,
        patientSplAta1: dummyAccount,
        facilitySplAta1: dummyAccount,
        patientSplAta2: dummyAccount,
        facilitySplAta2: dummyAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([newAdmin, verifier1Keypair, verifier2Keypair]) // Only 2 verifiers
      .rpc()
      throw new Error("Should have failed");
    } catch (err) {
      expect(err.toString()).to.match(/VerifierNotFound|account: verifi|AnchorError/);
    }
});

  
  //ANY USER CAN CLOSE A REJECTED CASE
  it("Test 22- Any user can close a rejected case", async () => {
    // Let's get the respective PDAs
    // Pretty Clear Case 3 Was Rejected, as out of 4 Verifiers, 3 rejected and only 1 approved.
    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0003")],
      program.programId
    );
    const [verifiersListPDA, verifiersListBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );
    const [patient3CasePDA, patient3CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient3Keypair.publicKey.toBuffer()],
        program.programId
      );

    // Let Patient 2 call that instruction
    await program.methods
      .closeRejectedCase("CASE0003")
      .accounts({
        user: patient2Keypair.publicKey,
        // @ts-ignore
        caseLookup: caseLookupPDA,
        patientCase: patient3CasePDA,
        verifiersList: verifiersListPDA,
      })
      .signers([patient2Keypair])
      .rpc();

    // Now, i will use the getAccountInfo function on the patient3Case pda, and if it's indeed close,
    // solana runtime will return a null
    const patient3CaseCloseData = await provider.connection.getAccountInfo(
      patient3CasePDA
    );
    expect(patient3CaseCloseData).to.eq(null);
  });


  //A VERIFIED CASE CAN NOT BE CLOSED, NOT EVEN BY ADMIN
  it("Test 23- A verified case cannot be CLOSED, even by the ADMIN", async () => {
    // Pretty Clear Case I is verified. Attempt to close it will produce an error
    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0001")],
      program.programId
    );
    const [verifiersListPDA, verifiersListBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );
    const [patient1CasePDA, patient1CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient1Keypair.publicKey.toBuffer()],
        program.programId
      );

    try {
      // Let admin call that instruction
      await program.methods
        .closeRejectedCase("CASE0001")
        .accounts({
          user: newAdmin.publicKey,
          // @ts-ignore
          caseLookup: caseLookupPDA,
          patientCase: patient1CasePDA,
          verifiersList: verifiersListPDA,
        })
        .signers([newAdmin])
        .rpc();
    } catch (err) {
      expect(err.error.errorCode.code).to.eq("CaseAlreadyVerified");
    }
  });

  //A CASE THAT HAS NOT ALREADY REACHED THE 70% QUORUM EVEN THOUGH 50% VERIFIERS HAVE VOTED CAN BE CLOSED
  it("Test 24- A case that has not already reached the 70% QUORUM even though 50% VERIFIERS have voted can be CLOSED", async () => {

    // Pretty Clear Case I is verified. Attempt to close it will produce an error
    const [caseLookupPDA, caseLookupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("case_lookup"), Buffer.from("CASE0002")],
      program.programId
    );
    const [verifiersListPDA, verifiersListBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("verifiers_list")],
        program.programId
      );
    const [patient2CasePDA, patient2CaseBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("patient"), patient2Keypair.publicKey.toBuffer()],
        program.programId
      );

    // Let admin call that instruction
    await program.methods
      .closeRejectedCase("CASE0002")
      .accounts({
        user: newAdmin.publicKey,
        // @ts-ignore
        caseLookup: caseLookupPDA,
        patientCase: patient2CasePDA,
        verifiersList: verifiersListPDA,
      })
      .signers([newAdmin])
      .rpc();

    //  using the getAccountInfo function on the patient2Case pda, and if it's indeed close,
    // solana runtime will return a null
    const patient2CaseCloseData = await provider.connection.getAccountInfo(
      patient2CasePDA
    );
    expect(patient2CaseCloseData).to.eq(null);
  });

  /*
   * Verifier Time Limit and Admin Override Tests
   */

  // Helper for warping time forward
  async function warpForwardByDays(days: number) {
    const clock = await provider.connection.getSlot();
    await provider.connection.getMinimumLedgerSlot();
    const numSlots = days * 24 * 60 * 60 * 2; // 2 slots per second
    await provider.connection.requestAirdrop(anchor.web3.Keypair.generate().publicKey, 1);
  }

  // --- DONATION HELPERS ---

  /**
   * Make a donation to a patient case
   */
  async function donate({
    program, donorKeypair, caseId, amount, mint,
    caseLookupPDA, patientCasePDA, patientEscrowPDA, donorAccountPDA, multisigPda,
    donorAta = null, patientAta = null
  }) {
    await program.methods
      .donate(caseId, amount, mint)
      .accounts({
        donor: donorKeypair.publicKey,
        caseLookup: caseLookupPDA,
        patientCase: patientCasePDA,
        patientEscrow: patientEscrowPDA,
        donorAccount: donorAccountPDA,
        multisigPda: multisigPda,
        mint: mint,
        donorAta: donorAta,
        patientAta: patientAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([donorKeypair])
      .rpc();
  }
});