import {pipe, ProgramDerivedAddress,} from "@solana/web3.js";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {describe} from "mocha";
import {assert} from "chai";
import {
    CpmmTestingEnvironment,
    createTransaction, getTransactionLogs,
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
        const {program, programDataAddress, rpcClient, rent, headAuthority, owner, ammsConfigsManagerAuthority, user} = cpmmTestingEnvironment;

        /// Initialization

        it("Unauthorized attempt to initialize AmmsConfigsManager should fail", async () => {
            const input: InitializeAmmsConfigsManagerInput = {
                signer: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                programData: programDataAddress,
                cpmmProgram: program.CPMM_PROGRAM_ADDRESS,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized attempt of AmmsConfigsManager initialization");
                },
                (_error) => {}
            ));
        })

        it("Initialization of AmmsConfigsManager should fail with an invalid head authority", async () => {
            const input: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: user.address,
                programData: programDataAddress,
                cpmmProgram: program.CPMM_PROGRAM_ADDRESS,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected to fail initialization of AmmsConfigsManager with an invalid head authority");
                },
                (_error) => {}
            ));
        })

        it("Initialize AmmsConfigsManager", async () => {
            const input: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                programData: programDataAddress,
                cpmmProgram: program.CPMM_PROGRAM_ADDRESS,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).catch((error) => console.log(error));

            const ammsConfigsManagerAccount = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccount, "AmmsConfigsManager account was not created");
            assert.strictEqual(ammsConfigsManagerAccount.data.authority, ammsConfigsManagerAuthority.address, "Authority does not match the expected address");
            assert.strictEqual(ammsConfigsManagerAccount.data.headAuthority, owner.address, "Head authority does not match the expected owner address");
            assert.strictEqual(ammsConfigsManagerAccount.data.configsCount, BigInt(0), "Configs count should be initialized to 0");
            assert.strictEqual(ammsConfigsManagerAccount.data.bump, ammsConfigsManagerAddress[1].valueOf(), "Bump value is incorrect");
        })

        it("Reinitialization of AmmsConfigsManager should fail", async () => {
            const input: InitializeAmmsConfigsManagerInput = {
                signer: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                authority: ammsConfigsManagerAuthority.address,
                headAuthority: owner.address,
                programData: programDataAddress,
                cpmmProgram: program.CPMM_PROGRAM_ADDRESS,
                rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS
            };

            const ix = getInitializeAmmsConfigsManagerInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of reinitialization AmmsConfigsManager");
                },
                (_error) => {}
            ));
        })

        // Authority update

        it("Unauthorized attempt to update AmmsConfigsManager authority should fail", async () => {
            const input: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized update of AmmsConfigsManager authority");
                },
                (_error) => {}
            ));
        })

        it("Update AmmsConfigsManager authority by authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "AmmsConfigsManager doesn't exist");

            const input: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, user.address, "Authority was not updated to the expected user address");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, ammsConfigsManagerAccountBefore.data.headAuthority, "Head authority should remain unchanged");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count should remain unchanged after update");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump value should remain the same");
        })

        it("Update AmmsConfigsManager authority by head authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "AmmsConfigsManager doesn't exist");

            const input: UpdateAmmsConfigsManagerAuthorityInput = {
                authority: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newAuthority: ammsConfigsManagerAuthority.address
            };

            const ix = getUpdateAmmsConfigsManagerAuthorityInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, ammsConfigsManagerAuthority.address, "Authority was not updated to the expected authority address");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, ammsConfigsManagerAccountBefore.data.headAuthority, "Head authority should remain unchanged");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count should remain unchanged after update");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump value should remain the same");
        })

        /// Head authority update

        it("Unauthorized attempt to update AmmsConfigsManager head authority should fail", async () => {
            const input: UpdateAmmsConfigsManagerHeadAuthorityInput = {
                headAuthority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newHeadAuthority: user.address
            };

            const ix = getUpdateAmmsConfigsManagerHeadAuthorityInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized update of AmmsConfigsManager head authority");
                },
                (_error) => {}
            ));
        })

        it("Update AmmsConfigsManager head authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigsManagerAccountBefore, "AmmsConfigsManager doesn't exist");

            const input: UpdateAmmsConfigsManagerHeadAuthorityInput = {
                headAuthority: owner,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                newHeadAuthority: headAuthority.address
            };

            const ix = getUpdateAmmsConfigsManagerHeadAuthorityInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            )

            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.strictEqual(ammsConfigsManagerAccountAfter.data.authority, ammsConfigsManagerAccountBefore.data.authority, "Authority should remain unchanged");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.headAuthority, headAuthority.address, "Head authority was not updated to the expected authority address");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount, ammsConfigsManagerAccountBefore.data.configsCount, "Configs count should remain unchanged after update");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.bump, ammsConfigsManagerAccountBefore.data.bump, "Bump value should remain the same");
        })
    })
}