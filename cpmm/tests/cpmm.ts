import {ammsConfigsManagerTests} from "./amms-configs-manager.test";
import {
    getProgramDerivedAddress,
    generateKeyPairSigner,
    KeyPairSigner,
    getU64Encoder,
    Endian,
    getAddressEncoder,
    ProgramDerivedAddress
} from "@solana/web3.js";
import {before} from "mocha";
import {CpmmTestingEnvironment, createCpmmTestingEnvironment} from "./helpers";
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

    ammsConfigsManagerAddress = await getProgramDerivedAddress({programAddress, seeds: ["amms_configs_manager"]});
    console.log("AMMs Configs Manager PDA:", ammsConfigsManagerAddress);

    ammsConfigAddress = await getProgramDerivedAddress({programAddress, seeds: ["amms_config", getU64Encoder({ endian: Endian.Big }).encode(0)]});
    console.log("AMMs Config PDA:", ammsConfigAddress);

    lpMint = await generateKeyPairSigner();
    console.log("LP Mint Address:", lpMint.address);

    cpAmmAddress = await getProgramDerivedAddress({programAddress, seeds: ["cp_amm", getAddressEncoder().encode(lpMint.address)]});
    console.log("CP AMM PDA:", cpAmmAddress);
})
it("Cpmm program instructions tests", async () => {
    ammsConfigsManagerTests(cpmmTestingEnvironment, ammsConfigsManagerAddress);
    ammsConfigTests(cpmmTestingEnvironment, ammsConfigsManagerAddress, ammsConfigAddress);
    cpAmmTests(cpmmTestingEnvironment, ammsConfigAddress, lpMint, cpAmmAddress);
});