import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";
import {ammsConfigsManagerTests} from "./amms-configs-manager.test";
import {
    getProgramDerivedAddress,
    address,
    generateKeyPairSigner,
    KeyPairSigner,
    createSolanaRpc,
    createSolanaRpcSubscriptions,
    getU64Encoder,
    Endian,
    getAddressEncoder,
    ProgramDerivedAddress
} from "@solana/web3.js";
import {describe, before} from "mocha";
import {CpmmTestingEnvironment} from "./helpers";
import {ammsConfigTests} from "./amms-config.test";
import {cpAmmTests} from "./cp-amm.test";

anchor.setProvider(anchor.AnchorProvider.env());


const httpEndpoint = 'http://127.0.0.1:8899';
const wsEndpoint = 'ws://127.0.0.1:8900';
describe("Cpmm program instructions tests", () => {
    const cpmmTestingEnvironment: CpmmTestingEnvironment = {
        program: anchor.workspace.Cpmm as Program<Cpmm>,
        rpc: createSolanaRpc(httpEndpoint),
        rpcSubscriptions: createSolanaRpcSubscriptions(wsEndpoint)
    };

    let ammsConfigsManagerAddress: ProgramDerivedAddress;
    let ammsConfigAddress: ProgramDerivedAddress;
    let lpMint: KeyPairSigner;
    let cpAmmAddress: ProgramDerivedAddress;

    before(async () =>{
        const programAddress = address(cpmmTestingEnvironment.program.programId.toBase58());
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

    ammsConfigsManagerTests(cpmmTestingEnvironment, ammsConfigsManagerAddress);
    ammsConfigTests(cpmmTestingEnvironment, ammsConfigsManagerAddress, ammsConfigAddress);
    cpAmmTests(cpmmTestingEnvironment, ammsConfigAddress, lpMint, cpAmmAddress);
})