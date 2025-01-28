import {KeyPairSigner, ProgramDerivedAddress} from "@solana/web3.js";
import {before, describe} from "mocha";
import {CpmmTestingEnvironment, getTestUser} from "./helpers";
export const ammsConfigTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigsManagerAddress: ProgramDerivedAddress, ammsConfigAddress: ProgramDerivedAddress) =>{
    describe("\nAmmsConfig tests", () =>{
        let unauthorizedUser: KeyPairSigner;
        before(async () =>{
            unauthorizedUser = await getTestUser(cpmmTestingEnvironment.rpc, cpmmTestingEnvironment.rpcSubscriptions, 100);
        })

        it("Initialize AmmsConfig", async () => {

        })
    })
}