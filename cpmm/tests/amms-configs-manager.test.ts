import {pipe, ProgramDerivedAddress,} from "@solana/web3.js";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {describe} from "mocha";
import {assert} from "chai";
import {
    CpmmTestingEnvironment,
    createTransaction,
    signAndSendTransaction
} from "./helpers";
import {
    getInitializeAmmsConfigsManagerInstruction,
    getUpdateAmmsConfigsManagerAuthorityInstruction, getUpdateAmmsConfigsManagerHeadAuthorityInstruction,
    InitializeAmmsConfigsManagerInput,
    UpdateAmmsConfigsManagerAuthorityInput,
    UpdateAmmsConfigsManagerHeadAuthorityInput
} from "../clients/js/src/generated";

export const ammsConfigsManagerTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigsManagerAddress: ProgramDerivedAddress) =>{
    describe("\nAmmsConfigsManager tests", () =>{
        const {program, rpcClient, rent, headAuthority, owner, ammsConfigsManagerAuthority, user} = cpmmTestingEnvironment;
        it("Unauthorized attempt to initialize AmmsConfigsManager should fail", async () => {
            const accounts: InitializeAmmsConfigsManagerInput = {
                signer: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(accounts);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of unauthorized attempt of initialization AmmsConfigsManager")).catch();
        })

        it("Initialization of AmmsConfigsManager should fail with an invalid head authority", async () => {
            const accounts: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: user.address,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(accounts);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected to fail initialization of AmmsConfigsManager with an invalid head authority")).catch();
        })

        it("Initialize AmmsConfigsManager", async () => {
            const accounts: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(accounts);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigsManagerAccount = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccount, "Account should exist");
            assert.strictEqual(ammsConfigsManagerAccount.data.authority, ammsConfigsManagerAuthority.address, "Authority is incorrect");
            assert.strictEqual(ammsConfigsManagerAccount.data.headAuthority, owner.address, "Head authority is incorrect");
            assert.strictEqual(ammsConfigsManagerAccount.data.configsCount, BigInt(0), "Configs count should start at 0");
            assert.strictEqual(ammsConfigsManagerAccount.data.bump, ammsConfigsManagerAddress[1].valueOf(), "Bump should be a valid number");
        })

        it("Reinitialization of AmmsConfigsManager should fail", async () => {
            const accounts: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(accounts);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => {
                assert.fail("Expected failure of reinitialization AmmsConfigsManager");
            }).catch();
        })

        it("Unauthorized attempt to update AmmsConfigsManager authority should fail", async () => {
            const accounts: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(accounts);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of unauthorized update of AmmsConfigsManager authority")).catch();
        })

        it("Update AmmsConfigsManager authority by authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "Account should exist");

            const accounts: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(accounts);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, user.address, "Authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, ammsConfigsManagerAccountBefore.data.headAuthority, "Head authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump mismatch");
        })

        it("Update AmmsConfigsManager authority by head authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "Account should exist");

            const accounts: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: ammsConfigsManagerAuthority.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(accounts);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, ammsConfigsManagerAuthority.address, "Authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, ammsConfigsManagerAccountBefore.data.headAuthority, "Head authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump mismatch");
        })

        it("Unauthorized attempt to update AmmsConfigsManager head authority should fail", async () => {
            const accounts: UpdateAmmsConfigsManagerHeadAuthorityInput = {
                headAuthority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newHeadAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerHeadAuthorityInstruction(accounts);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of unauthorized update of AmmsConfigsManager head authority")).catch();
        })

        it("Update AmmsConfigsManager head authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "Account should exist");

            const accounts: UpdateAmmsConfigsManagerHeadAuthorityInput = {
                headAuthority: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newHeadAuthority: headAuthority.address
            };

            const ix = getUpdateAmmsConfigsManagerHeadAuthorityInstruction(accounts);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, ammsConfigsManagerAccountBefore.data.authority, "Authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, headAuthority.address, "Head authority mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count mismatch");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump mismatch");
        })
    })
}