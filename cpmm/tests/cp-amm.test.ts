import {KeyPairSigner, ProgramDerivedAddress} from "@solana/web3.js";
import {before, describe} from "mocha";
import {CpmmTestingEnvironment, getTestUser} from "./helpers";
export const cpAmmTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigAddress: ProgramDerivedAddress, lpMint: KeyPairSigner, cpAmm: ProgramDerivedAddress) =>{
    describe("\nCpAmm tests", () =>{
        let unauthorizedUser: KeyPairSigner;
        before(async () =>{
            unauthorizedUser = await getTestUser(cpmmTestingEnvironment.rpc, cpmmTestingEnvironment.rpcSubscriptions, 100);
        })
        it("Initialize CpAmm", async () => {

        })
    })
}
