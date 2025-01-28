import {KeyPairSigner, ProgramDerivedAddress, } from "@solana/web3.js";
import {web3} from "@coral-xyz/anchor";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {before, describe} from "mocha";
import {CpmmTestingEnvironment, getTestUser} from "./helpers";
export const ammsConfigsManagerTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigsManagerAddress: ProgramDerivedAddress) =>{
    describe("\nAmmsConfigsManager tests", () =>{
        let authority: KeyPairSigner;
        let unauthorizedUser: KeyPairSigner;
        before(async () =>{
            authority = await getTestUser(cpmmTestingEnvironment.rpc, cpmmTestingEnvironment.rpcSubscriptions, 100);
            unauthorizedUser = await getTestUser(cpmmTestingEnvironment.rpc, cpmmTestingEnvironment.rpcSubscriptions, 100);
        })

        it("Initialize AmmsConfigsManager", async () => {
            try {
                const accounts = {
                    signer: cpmmTestingEnvironment.program.provider.publicKey!,
                    amms_configs_manager: ammsConfigsManagerAddress,
                    authority: authority.address,
                    head_authority: cpmmTestingEnvironment.program.provider.publicKey!,
                    rent: web3.SYSVAR_RENT_PUBKEY,
                    system_program: SYSTEM_PROGRAM_ADDRESS
                };
                await cpmmTestingEnvironment.program.methods.initializeAmmsConfigsManager().accounts(accounts).signers([]).rpc();
            } catch (err) {
                console.error("Failed to initialize AmmsConfigsManager:", err);
            }
        })

    })
}