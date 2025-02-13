import {
    Endian,
    getProgramDerivedAddress,
    getU64Encoder,
    KeyPairSigner,
    pipe,
    ProgramDerivedAddress
} from "@solana/web3.js";
import {before, describe} from "mocha";
import {
    CpmmTestingEnvironment,
    createTransaction,
    createTestUser,
    signAndSendTransaction,
    getTransactionLogs
} from "./helpers";
import {
    getInitializeAmmsConfigInstruction,
    getUpdateAmmsConfigFeeAuthorityInstruction,
    getUpdateAmmsConfigProtocolFeeRateInstruction,
    getUpdateAmmsConfigProvidersFeeRateInstruction,
    InitializeAmmsConfigInput,
    UpdateAmmsConfigFeeAuthorityInput,
    UpdateAmmsConfigProtocolFeeRateInput,
    UpdateAmmsConfigProvidersFeeRateInput
} from "../clients/js/src/generated";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {assert} from "chai";
export const ammsConfigTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigsManagerAddress: ProgramDerivedAddress, ammsConfigAddress: ProgramDerivedAddress) =>{
    describe("\nAmmsConfig tests", () =>{
        const {program, rpcClient, rent, headAuthority, owner, ammsConfigsManagerAuthority, user} = cpmmTestingEnvironment;
        let feeAuthority: KeyPairSigner;
        let malwareAmmsConfigsManagerAddress: ProgramDerivedAddress;
        before(async () =>{
            feeAuthority = await createTestUser(rpcClient, 100);
            malwareAmmsConfigsManagerAddress = await getProgramDerivedAddress({
                programAddress: program.CPMM_PROGRAM_ADDRESS,
                seeds: ["ammss_configs_manager"]
            });
        })

        /// Initialization

        it("Unauthorized attempt to initialize AmmsConfig should fail", async () => {
            const input: InitializeAmmsConfigInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints: 40,
                providersFeeRateBasisPoints: 75
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized attempt of AmmsConfig initialization");
                },
                (_error) => {}
            ));
        })

        it("Initialization of AmmsConfig with exceeded fees should fail", async () => {
            const input: InitializeAmmsConfigInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints: 5001,
                providersFeeRateBasisPoints: 5000
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of AmmsConfig initialization with exceeded fees");
                },
                (_error) => {}
            ));
        })

        it("Initialization of AmmsConfig with malware AmmsConfigManager should fail", async () => {

            const input: InitializeAmmsConfigInput = {
                authority: user,
                ammsConfigsManager: malwareAmmsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints: 5001,
                providersFeeRateBasisPoints: 5000
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of AmmsConfig initialization with malware AmmsConfigManager");
                },
                (_error) => {}
            ));
        })

        it("Initialize AmmsConfig by head authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);
            assert.ok(ammsConfigsManagerAccountBefore, "AmmsConfigsManager doesn't exist");

            const protocolFeeRateBasisPoints = 40;
            const providersFeeRateBasisPoints = 75;

            const input: InitializeAmmsConfigInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints,
                providersFeeRateBasisPoints
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccount = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(ammsConfigAccount, "AmmsConfig account was not created");
            assert.strictEqual(ammsConfigAccount.data.feeAuthority, feeAuthority.address, "Fee authority does not match expected value");
            assert.strictEqual(ammsConfigAccount.data.id, ammsConfigsManagerAccountBefore.data.configsCount, "Config ID does not match expected count");
            assert.strictEqual(ammsConfigAccount.data.protocolFeeRateBasisPoints, protocolFeeRateBasisPoints, "Protocol fee rate is incorrect");
            assert.strictEqual(ammsConfigAccount.data.providersFeeRateBasisPoints, providersFeeRateBasisPoints, "Provider fee rate is incorrect");
            assert.strictEqual(ammsConfigAccount.data.bump, ammsConfigAddress[1].valueOf(), "Bump value is incorrect");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount - ammsConfigsManagerAccountBefore.data.configsCount, BigInt(1), "Configs count was not incremented correctly");
        })

        it("Initialize AmmsConfig by authority", async () => {
            const ammsConfigsManagerAccountBefore = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);
            assert.ok(ammsConfigsManagerAccountBefore, "AmmsConfigsManager doesn't exist");

            const protocolFeeRateBasisPoints = 40;
            const providersFeeRateBasisPoints = 75;

            const testAmmsConfigAddress = await getProgramDerivedAddress({
                programAddress: cpmmTestingEnvironment.program.CPMM_PROGRAM_ADDRESS,
                seeds: ["amms_config", getU64Encoder({ endian: Endian.Little }).encode(ammsConfigsManagerAccountBefore.data.configsCount)]
            });

            const input: InitializeAmmsConfigInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: testAmmsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints,
                providersFeeRateBasisPoints
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const testAmmsConfigAccount = await program.fetchAmmsConfig(rpcClient.rpc, testAmmsConfigAddress[0]);
            const ammsConfigsManagerAccountAfter = await program.fetchAmmsConfigsManager(rpcClient.rpc, ammsConfigsManagerAddress[0]);

            assert.ok(testAmmsConfigAccount, "AmmsConfig account was not created");
            assert.strictEqual(testAmmsConfigAccount.data.feeAuthority, feeAuthority.address, "Fee authority does not match expected value");
            assert.strictEqual(testAmmsConfigAccount.data.id, ammsConfigsManagerAccountBefore.data.configsCount, "Config ID does not match expected count");
            assert.strictEqual(testAmmsConfigAccount.data.protocolFeeRateBasisPoints, protocolFeeRateBasisPoints, "Protocol fee rate is incorrect");
            assert.strictEqual(testAmmsConfigAccount.data.providersFeeRateBasisPoints, providersFeeRateBasisPoints, "Provider fee rate is incorrect");
            assert.strictEqual(testAmmsConfigAccount.data.bump, testAmmsConfigAddress[1].valueOf(), "Bump value is incorrect");
            assert.strictEqual(ammsConfigsManagerAccountAfter.data.configsCount - ammsConfigsManagerAccountBefore.data.configsCount, BigInt(1), "Configs count was not incremented correctly");
        })

        it("Reinitialization of AmmsConfig should fail", async () => {

            const input: InitializeAmmsConfigInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                feeAuthority: feeAuthority.address,
                rent: rent,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                protocolFeeRateBasisPoints: 45,
                providersFeeRateBasisPoints: 75
            };

            const ix = getInitializeAmmsConfigInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of reinitialization AmmsConfig")).catch();
        })

        /// Fee authority update

        it("Unauthorized attempt to update AmmsConfig fee authority should fail", async () => {
            const input: UpdateAmmsConfigFeeAuthorityInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newFeeAuthority: cpmmTestingEnvironment.headAuthority.address
            };

            const ix = getUpdateAmmsConfigFeeAuthorityInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized update of AmmsConfig fee authority");
                },
                (_error) => {}
            ));
        })

        it("Update of AmmsConfig fee authority with malware AmmsConfigManager should fail", async () => {
            const input: UpdateAmmsConfigFeeAuthorityInput = {
                authority: headAuthority,
                ammsConfigsManager: malwareAmmsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newFeeAuthority: malwareAmmsConfigsManagerAddress[0]
            };

            const ix = getUpdateAmmsConfigFeeAuthorityInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of update of AmmsConfig fee authority with malware AmmsConfigManager");
                },
                (_error) => {}
            ));
        })

        it("Update AmmsConfig fee authority by head authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const input: UpdateAmmsConfigFeeAuthorityInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newFeeAuthority: user.address
            };

            const ix = getUpdateAmmsConfigFeeAuthorityInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, user.address, "Fee authority does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints,  ammsConfigAccountBefore.data.protocolFeeRateBasisPoints, "Protocol fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  ammsConfigAccountBefore.data.providersFeeRateBasisPoints, "Provider fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        it("Update AmmsConfig fee authority by authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const input: UpdateAmmsConfigFeeAuthorityInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newFeeAuthority: headAuthority.address
            };

            const ix = getUpdateAmmsConfigFeeAuthorityInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, headAuthority.address, "Fee authority does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints,  ammsConfigAccountBefore.data.protocolFeeRateBasisPoints, "Protocol fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  ammsConfigAccountBefore.data.providersFeeRateBasisPoints, "Provider fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        /// Protocol fee rate update

        it("Unauthorized attempt to update AmmsConfig protocol fee rate should fail", async () => {
            const input: UpdateAmmsConfigProtocolFeeRateInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProtocolFeeRateBasisPoints: 312
            };

            const ix = getUpdateAmmsConfigProtocolFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized update of AmmsConfig protocol fee rate");
                },
                (_error) => {}
            ));
        })

        it("Update of AmmsConfig protocol fee rate with malware AmmsConfigManager should fail", async () => {
            const input: UpdateAmmsConfigProtocolFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: malwareAmmsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProtocolFeeRateBasisPoints: 312
            };

            const ix = getUpdateAmmsConfigProtocolFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of update of AmmsConfig protocol fee rate with malware AmmsConfigManager");
                },
                (_error) => {}
            ));
        })

        it("Update AmmsConfig protocol fee rate by head authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const newProtocolFeeRateBasisPoints = 657;

            const input: UpdateAmmsConfigProtocolFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProtocolFeeRateBasisPoints
            };

            const ix = getUpdateAmmsConfigProtocolFeeRateInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, ammsConfigAccountBefore.data.feeAuthority, "Fee authority should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints, newProtocolFeeRateBasisPoints, "Protocol fee rate does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  ammsConfigAccountBefore.data.providersFeeRateBasisPoints, "Provider fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        it("Update AmmsConfig protocol fee rate by authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const newProtocolFeeRateBasisPoints = 100;

            const input: UpdateAmmsConfigProtocolFeeRateInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProtocolFeeRateBasisPoints
            };

            const ix = getUpdateAmmsConfigProtocolFeeRateInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, ammsConfigAccountBefore.data.feeAuthority, "Fee authority should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints, newProtocolFeeRateBasisPoints, "Protocol fee rate does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  ammsConfigAccountBefore.data.providersFeeRateBasisPoints, "Provider fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        it("Update AmmsConfig protocol fee rate to exceeding fee should fail", async () => {
            const input: UpdateAmmsConfigProtocolFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProtocolFeeRateBasisPoints: 9926
            };

            const ix = getUpdateAmmsConfigProtocolFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of update of AmmsConfig protocol fee rate to exceeding fee");
                },
                (_error) => {}
            ));
        })

        /// Providers fee rate update

        it("Unauthorized attempt to update AmmsConfig providers fee rate should fail", async () => {
            const input: UpdateAmmsConfigProvidersFeeRateInput = {
                authority: user,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProvidersFeeRateBasisPoints: 312
            };

            const ix = getUpdateAmmsConfigProvidersFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unauthorized update of AmmsConfig providers fee rate");
                },
                (_error) => {}
            ));
        })

        it("Update of AmmsConfig providers fee rate with malware AmmsConfigManager should fail", async () => {
            const input: UpdateAmmsConfigProvidersFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: malwareAmmsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProvidersFeeRateBasisPoints: 312
            };

            const ix = getUpdateAmmsConfigProvidersFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of update of AmmsConfig providers fee rate with malware AmmsConfigManager");
                },
                (_error) => {}
            ));
        })

        it("Update AmmsConfig providers fee rate by head authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const newProvidersFeeRateBasisPoints = 657;

            const input: UpdateAmmsConfigProvidersFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProvidersFeeRateBasisPoints
            };

            const ix = getUpdateAmmsConfigProvidersFeeRateInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, ammsConfigAccountBefore.data.feeAuthority, "Fee authority should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints, ammsConfigAccountBefore.data.protocolFeeRateBasisPoints, "Protocol fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  newProvidersFeeRateBasisPoints, "Provider fee rate does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        it("Update AmmsConfig providers fee rate by authority", async () => {
            const ammsConfigAccountBefore = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);
            assert.ok(ammsConfigAccountBefore, "AmmsConfig doesn't exist");

            const newProvidersFeeRateBasisPoints = 400;

            const input: UpdateAmmsConfigProvidersFeeRateInput = {
                authority: ammsConfigsManagerAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProvidersFeeRateBasisPoints
            };

            const ix = getUpdateAmmsConfigProvidersFeeRateInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const ammsConfigAccountAfter = await program.fetchAmmsConfig(rpcClient.rpc, ammsConfigAddress[0]);

            assert.strictEqual(ammsConfigAccountAfter.data.feeAuthority, ammsConfigAccountBefore.data.feeAuthority, "Fee authority should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.id, ammsConfigAccountBefore.data.id, "Config ID should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.protocolFeeRateBasisPoints, ammsConfigAccountBefore.data.protocolFeeRateBasisPoints, "Protocol fee rate should remain unchanged");
            assert.strictEqual(ammsConfigAccountAfter.data.providersFeeRateBasisPoints,  newProvidersFeeRateBasisPoints, "Provider fee rate does not match expected value");
            assert.strictEqual(ammsConfigAccountAfter.data.bump,  ammsConfigAccountBefore.data.bump, "Bump should remain unchanged");
        })

        it("Update AmmsConfig providers fee rate to exceeding fee should fail", async () => {
            const input: UpdateAmmsConfigProvidersFeeRateInput = {
                authority: headAuthority,
                ammsConfigsManager: ammsConfigsManagerAddress[0],
                ammsConfig: ammsConfigAddress[0],
                newProvidersFeeRateBasisPoints: 9901
            };

            const ix = getUpdateAmmsConfigProvidersFeeRateInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of update of AmmsConfig providers fee rate to exceeding fee");
                },
                (_error) => {}
            ));
        })
    })
}