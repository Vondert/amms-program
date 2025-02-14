import {ammsConfigsManagerTests} from "./amms-configs-manager.test";
import {
    generateKeyPairSigner,
    KeyPairSigner,
    ProgramDerivedAddress
} from "@solana/web3.js";
import {before} from "mocha";
import {
    CpmmTestingEnvironment,
    createCpmmTestingEnvironment,
    getAmmsConfigPDA,
    getAmmsConfigsManagerPDA, getCpAmmPDA
} from "./helpers";
import {ammsConfigTests} from "./amms-config.test";
import {cpAmmTests} from "./cp-amm.test";

// Declare testing environment and key variables
let cpmmTestingEnvironment: CpmmTestingEnvironment;
let ammsConfigsManagerAddress: ProgramDerivedAddress;
let ammsConfigAddress: ProgramDerivedAddress;
let lpMint: KeyPairSigner;
let cpAmmAddress: ProgramDerivedAddress;

/**
 * Setup test environment before running tests.
 * Initializes the CPMM testing environment and retrieves necessary program addresses.
 */
before(async () =>{
    // Initialize CPMM testing environment
    cpmmTestingEnvironment = await createCpmmTestingEnvironment();
    const programAddress = cpmmTestingEnvironment.program.CPMM_PROGRAM_ADDRESS;
    console.log("Program Address:", programAddress);

    // Fetch the AMMs Configs Manager PDA
    ammsConfigsManagerAddress = await getAmmsConfigsManagerPDA();
    console.log("AMMs Configs Manager PDA:", ammsConfigsManagerAddress);

    // Fetch a specific AMMs Config PDA (assuming config ID 0)
    ammsConfigAddress = await getAmmsConfigPDA(BigInt(0));
    console.log("AMMs Config PDA:", ammsConfigAddress);

    // Generate a key pair for LP Mint
    lpMint = await generateKeyPairSigner();
    console.log("LP Mint Address:", lpMint.address);

    // Fetch the PDA for a constant product AMM linked to LP Mint
    cpAmmAddress = await getCpAmmPDA(lpMint.address);
    console.log("CP AMM PDA:", cpAmmAddress);
});

/**
 * Runs a series of tests for the CPMM program instructions.
 * This includes testing AMMs Configs Manager, AMMs Config, and CP AMM functionalities.
 */
it("Cpmm program instructions tests", async () => {
    // Run tests for AMMs Configs Manager
    ammsConfigsManagerTests(cpmmTestingEnvironment, ammsConfigsManagerAddress);

    // Run tests for AMMs Config, linking it with the Configs Manager
    ammsConfigTests(cpmmTestingEnvironment, ammsConfigsManagerAddress, ammsConfigAddress);

    // Run tests for CP AMM, linking it with AMMs Config
    cpAmmTests(cpmmTestingEnvironment, ammsConfigAddress);
});
