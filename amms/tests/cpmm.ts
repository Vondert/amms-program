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

let cpmmTestingEnvironment: CpmmTestingEnvironment;
let ammsConfigsManagerAddress: ProgramDerivedAddress;
let ammsConfigAddress: ProgramDerivedAddress;
let lpMint: KeyPairSigner;
let cpAmmAddress: ProgramDerivedAddress;



before(async () =>{
    cpmmTestingEnvironment = await createCpmmTestingEnvironment();
    const programAddress = cpmmTestingEnvironment.program.CPMM_PROGRAM_ADDRESS;
    console.log("Program Address:", programAddress);

    ammsConfigsManagerAddress = await getAmmsConfigsManagerPDA();
    console.log("AMMs Configs Manager PDA:", ammsConfigsManagerAddress);

    ammsConfigAddress = await getAmmsConfigPDA(BigInt(0));
    console.log("AMMs Config PDA:", ammsConfigAddress);

    lpMint = await generateKeyPairSigner();
    console.log("LP Mint Address:", lpMint.address);

    cpAmmAddress = await getCpAmmPDA(lpMint.address);
    console.log("CP AMM PDA:", cpAmmAddress);
})
it("Cpmm program instructions tests", async () => {
    ammsConfigsManagerTests(cpmmTestingEnvironment, ammsConfigsManagerAddress);
    ammsConfigTests(cpmmTestingEnvironment, ammsConfigsManagerAddress, ammsConfigAddress);
    cpAmmTests(cpmmTestingEnvironment, ammsConfigAddress);
});