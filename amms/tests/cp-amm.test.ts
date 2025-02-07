import {
    Account, generateKeyPairSigner, KeyPairSigner, none,
    pipe, ProgramDerivedAddress, Some, some
} from "@solana/web3.js";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {
    ASSOCIATED_TOKEN_PROGRAM_ADDRESS, TOKEN_PROGRAM_ADDRESS, fetchMint, findAssociatedTokenPda,
    Mint as TokenMint, Token as TokenAccount,
} from "@solana-program/token";
import {
    Mint as Token22Mint, Token as Token22Account, fetchMint as fetchMint22, Extension,
} from "@solana-program/token-2022";
import {assert} from "chai";
import {before, describe} from "mocha";
import {
    fetchCpAmm,
    getInitializeCpAmmInstruction, getLaunchCpAmmInstruction, getProvideToCpAmmInstruction,
    InitializeCpAmmInput,
    LaunchCpAmmInput, ProvideToCpAmmInput
} from "../clients/js/src/generated";
import {
    CpmmTestingEnvironment,
    createTestUser,
    createTransaction, createTransactionWithComputeUnits,
    getCpAmmPDA, getToken22PDA, getTokenPDA, getTransactionLogs,
    signAndSendTransaction
} from "./helpers";
import {
    createAtaWithTokens, createAtaWithTokens22,
    createToken22Mint,
    createToken22MintWithPermanentDelegate,
    createToken22MintWithTransferFee,
    createTokenMint
} from "./tokens-helpers";


export const cpAmmTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigAddress: ProgramDerivedAddress) =>{
    describe("\nCpAmm tests", () =>{
        const {rpcClient, rent, headAuthority, owner, user} = cpmmTestingEnvironment;
        let generalUser: KeyPairSigner;

        const TEST_CP_AMMS: {
            lpMint1: KeyPairSigner,
            cpAmm1: ProgramDerivedAddress,
            baseVault1: ProgramDerivedAddress,
            quoteVault1: ProgramDerivedAddress,
            lpVault1: ProgramDerivedAddress,
            lpMint2: KeyPairSigner,
            cpAmm2: ProgramDerivedAddress,
            baseVault2: ProgramDerivedAddress,
            quoteVault2: ProgramDerivedAddress,
            lpVault2: ProgramDerivedAddress,
            lpMint3: KeyPairSigner,
            cpAmm3: ProgramDerivedAddress,
            baseVault3: ProgramDerivedAddress,
            quoteVault3: ProgramDerivedAddress,
            lpVault3: ProgramDerivedAddress
        } = {
            lpMint1: undefined,
            cpAmm1: undefined,
            baseVault1: undefined,
            quoteVault1: undefined,
            lpVault1: undefined,
            lpMint2: undefined,
            cpAmm2: undefined,
            baseVault2: undefined,
            quoteVault2: undefined,
            lpVault2: undefined,
            lpMint3: undefined,
            cpAmm3: undefined,
            baseVault3: undefined,
            quoteVault3: undefined,
            lpVault3: undefined,
        };
        const TEST_MINTS: {
            validTokenMint1: Account<TokenMint>,
            validTokenMint2: Account<TokenMint>,
            validTokenMint3: Account<TokenMint>,
            freezeAuthorityTokenMint: Account<TokenMint>,
            validToken22Mint1: Account<Token22Mint>,
            validToken22Mint2: Account<Token22Mint>,
            transferFeeToken2022Mint: Account<Token22Mint>,
            permanentDelegateToken2022Mint: Account<Token22Mint>
        } = {
            validTokenMint1: undefined,
            validTokenMint2: undefined,
            validTokenMint3: undefined,
            freezeAuthorityTokenMint: undefined,
            validToken22Mint1: undefined,
            validToken22Mint2: undefined,
            transferFeeToken2022Mint: undefined,
            permanentDelegateToken2022Mint: undefined
        };
        const USER_TOKEN_ACCOUNTS: {
            validToken1: Account<TokenAccount>,
            validToken2: Account<TokenAccount>,
            validToken3: Account<TokenAccount>,
            validToken221: Account<Token22Account>,
            validToken222: Account<Token22Account>,
            transferFeeToken22: Account<Token22Account>,
            lpToken1: ProgramDerivedAddress,
            lpToken2: ProgramDerivedAddress,
            lpToken3: ProgramDerivedAddress
        } = {
            validToken1: undefined,
            validToken2: undefined,
            validToken3: undefined,
            validToken221: undefined,
            validToken222: undefined,
            transferFeeToken22: undefined,
            lpToken1: undefined,
            lpToken2: undefined,
            lpToken3: undefined
        };
        const GENERAL_USER_TOKEN_ACCOUNTS: {
            validToken1: Account<TokenAccount>,
            validToken2: Account<TokenAccount>,
            validToken3: Account<TokenAccount>,
            validToken221: Account<Token22Account>,
            validToken222: Account<Token22Account>,
            transferFeeToken22: Account<Token22Account>,
            lpToken1: ProgramDerivedAddress,
            lpToken2: ProgramDerivedAddress,
            lpToken3: ProgramDerivedAddress
        } = {
            validToken1: undefined,
            validToken2: undefined,
            validToken3: undefined,
            validToken221: undefined,
            validToken222: undefined,
            transferFeeToken22: undefined,
            lpToken1: undefined,
            lpToken2: undefined,
            lpToken3: undefined
        };

        before(async () =>{
            generalUser = await createTestUser(rpcClient, 100);

            [TEST_CP_AMMS.lpMint1, TEST_CP_AMMS.lpMint2, TEST_CP_AMMS.lpMint3] = await Promise.all([
                generateKeyPairSigner(),
                generateKeyPairSigner(),
                generateKeyPairSigner()
            ]);
            [TEST_CP_AMMS.cpAmm1, TEST_CP_AMMS.cpAmm2, TEST_CP_AMMS.cpAmm3] = await Promise.all([
                getCpAmmPDA(TEST_CP_AMMS.lpMint1.address),
                getCpAmmPDA(TEST_CP_AMMS.lpMint2.address),
                getCpAmmPDA(TEST_CP_AMMS.lpMint3.address)
            ]);

            TEST_MINTS.validTokenMint1 = await createTokenMint(rpcClient, user, 6);
            TEST_MINTS.validTokenMint2 = await createTokenMint(rpcClient, user, 4);
            TEST_MINTS.validTokenMint3 = await createTokenMint(rpcClient, user, 9);
            TEST_MINTS.freezeAuthorityTokenMint = await createTokenMint(rpcClient, user, 1, user.address);

            TEST_MINTS.validToken22Mint1 = await createToken22Mint(rpcClient, user, 3);
            TEST_MINTS.validToken22Mint2 = await createToken22Mint(rpcClient, user, 0);
            TEST_MINTS.transferFeeToken2022Mint = await createToken22MintWithTransferFee(rpcClient, user, 2, 379, 10000);
            TEST_MINTS.permanentDelegateToken2022Mint = await createToken22MintWithPermanentDelegate(rpcClient, user, 0);

            [
                TEST_CP_AMMS.baseVault1, TEST_CP_AMMS.baseVault2, TEST_CP_AMMS.baseVault3,
                TEST_CP_AMMS.quoteVault1, TEST_CP_AMMS.quoteVault2, TEST_CP_AMMS.quoteVault3,
                TEST_CP_AMMS.lpVault1, TEST_CP_AMMS.lpVault2, TEST_CP_AMMS.lpVault3,
                USER_TOKEN_ACCOUNTS.lpToken1, USER_TOKEN_ACCOUNTS.lpToken2, USER_TOKEN_ACCOUNTS.lpToken3,
                GENERAL_USER_TOKEN_ACCOUNTS.lpToken1, GENERAL_USER_TOKEN_ACCOUNTS.lpToken2, GENERAL_USER_TOKEN_ACCOUNTS.lpToken3,
            ] = await Promise.all([
                getTokenPDA(TEST_MINTS.validTokenMint1.address, TEST_CP_AMMS.cpAmm1[0]), getTokenPDA(TEST_MINTS.validTokenMint2.address, TEST_CP_AMMS.cpAmm2[0]), getTokenPDA(TEST_MINTS.validTokenMint2.address, TEST_CP_AMMS.cpAmm3[0]),
                getToken22PDA(TEST_MINTS.validToken22Mint1.address, TEST_CP_AMMS.cpAmm1[0]), getTokenPDA(TEST_MINTS.validTokenMint3.address, TEST_CP_AMMS.cpAmm2[0]), getToken22PDA(TEST_MINTS.transferFeeToken2022Mint.address, TEST_CP_AMMS.cpAmm3[0]),
                getTokenPDA(TEST_CP_AMMS.lpMint1.address, TEST_CP_AMMS.cpAmm1[0]), getTokenPDA(TEST_CP_AMMS.lpMint2.address, TEST_CP_AMMS.cpAmm2[0]), getTokenPDA(TEST_CP_AMMS.lpMint3.address, TEST_CP_AMMS.cpAmm3[0]),
                getTokenPDA(TEST_CP_AMMS.lpMint1.address, user.address), getTokenPDA(TEST_CP_AMMS.lpMint2.address, user.address), getTokenPDA(TEST_CP_AMMS.lpMint3.address, user.address),
                getTokenPDA(TEST_CP_AMMS.lpMint1.address, generalUser.address), getTokenPDA(TEST_CP_AMMS.lpMint2.address, generalUser.address), getTokenPDA(TEST_CP_AMMS.lpMint3.address,generalUser.address)
            ]);

            USER_TOKEN_ACCOUNTS.validToken1 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint1.address, user, user, BigInt(1_000_000_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken2 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint2.address, user, user, BigInt(1_000_000_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken3 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint3.address, user, user, BigInt(1_000_000_000_000_000));

            USER_TOKEN_ACCOUNTS.validToken221 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint1.address, user, user, BigInt(1_000_000_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken222 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint2.address, user, user, BigInt(1_000_000_000_000_000));
            USER_TOKEN_ACCOUNTS.transferFeeToken22 = await createAtaWithTokens22(rpcClient, TEST_MINTS.transferFeeToken2022Mint.address, user, user, BigInt(1_000_000_000_000_000));

            GENERAL_USER_TOKEN_ACCOUNTS.validToken1 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint1.address, user, generalUser, BigInt(1_000_000_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken2 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint2.address, user, generalUser, BigInt(1_000_000_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken3 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint3.address, user, generalUser, BigInt(1_000_000_000_000_000));

            GENERAL_USER_TOKEN_ACCOUNTS.validToken221 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint1.address, user, generalUser, BigInt(1_000_000_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken222 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint2.address, user, generalUser, BigInt(1_000_000_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.transferFeeToken22 = await createAtaWithTokens22(rpcClient, TEST_MINTS.transferFeeToken2022Mint.address, user, generalUser, BigInt(1_000_000_000_000_000));
        })

        // Initialize CpAmm

        it("Unfunded with 0.1 SOL CpAmm initialization attempt should fail", async () => {
            const unfundedUser = await createTestUser(rpcClient, 0.1);

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: unfundedUser,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of unfunded with 0.1 SOL CpAmm initialization attempt");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with equal mints should fail", async () => {

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization with equal mints");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with invalid fee authority should fail", async () => {

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: user.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization with invalid fee authority");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with malware AmmsConfig should fail", async () => {
            const malwareAmmsConfigAddress = TEST_MINTS.validTokenMint2.address;
            const input: InitializeCpAmmInput = {
                ammsConfig: malwareAmmsConfigAddress,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization malware AmmsConfig");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with invalid LP mint should fail", async () => {
            const invalidLpMint = await generateKeyPairSigner();
            const [ata] = await findAssociatedTokenPda({
                mint: invalidLpMint.address,
                owner: user.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: invalidLpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: ata,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization with invalid LP mint");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with mint with freeze authority should fail", async () => {

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.freezeAuthorityTokenMint.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization with mint with freeze authority");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with mint with one of forbidden token extensions (Permanent Delegate) should fail", async () => {
            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.permanentDelegateToken2022Mint.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm initialization with mint with one of forbidden token extensions (Permanent Delegate)");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with token mint and token 2022", async () => {
            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validToken22Mint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const [feeAuthorityBalanceAfter, lpMintAccount, cpAmmAccount] = await Promise.all([
                rpcClient.rpc.getBalance(headAuthority.address).send(),
                fetchMint(rpcClient.rpc, TEST_CP_AMMS.lpMint1.address),
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.creator, user.address,  "Creator address mismatch");
            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint1.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.validToken22Mint1.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "CpAmm shouldn't be launched");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], TEST_CP_AMMS.cpAmm1[1].valueOf(), "Bump value is incorrect");
        })

        it("Reinitialization of CpAmm should fail", async () => {
            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: TEST_CP_AMMS.cpAmm1[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint1,
                quoteMint: TEST_MINTS.validToken22Mint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken1[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm reinitialization");
                },
                (_error) => {}
            ));
        })

        it("Initialization CpAmm with two token mints", async () => {
            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint2.address,
                cpAmm: TEST_CP_AMMS.cpAmm2[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint2,
                quoteMint: TEST_MINTS.validTokenMint3.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken2[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const [feeAuthorityBalanceAfter, lpMintAccount, cpAmmAccount] = await Promise.all([
                rpcClient.rpc.getBalance(headAuthority.address).send(),
                fetchMint(rpcClient.rpc, TEST_CP_AMMS.lpMint2.address),
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm2[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.creator, user.address,  "Creator address mismatch");
            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint2.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.validTokenMint3.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "CpAmm shouldn't be launched");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], TEST_CP_AMMS.cpAmm2[1].valueOf(), "Bump value is incorrect");
        })

        it("Initialization CpAmm with token mint and token 2022 with one of allowed extensions (Transfer Fee Config)", async () => {
            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            const input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint2.address,
                cpAmm: TEST_CP_AMMS.cpAmm3[0],
                feeAuthority: headAuthority.address,
                lpMint: TEST_CP_AMMS.lpMint3,
                quoteMint: TEST_MINTS.transferFeeToken2022Mint.address,
                rent,
                signer: user,
                signerLpTokenAccount: USER_TOKEN_ACCOUNTS.lpToken3[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const [feeAuthorityBalanceAfter, lpMintAccount, cpAmmAccount] = await Promise.all([
                rpcClient.rpc.getBalance(headAuthority.address).send(),
                fetchMint(rpcClient.rpc, TEST_CP_AMMS.lpMint3.address),
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm3[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.creator, user.address,  "Creator address mismatch");
            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint2.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.transferFeeToken2022Mint.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "CpAmm shouldn't be launched");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], TEST_CP_AMMS.cpAmm3[1].valueOf(), "Bump value is incorrect");
        })

        // Launch CpAmm

        it("Launching CpAmm with insufficient balance of base tokens on signer's account should fail", async () => {
            const cpAmmAccountBefore = await fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(9_000_000_000_000_000);
            const quoteLiquidity = BigInt(43241);

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault1[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault1[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault1[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken1.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken1[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken221.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await (pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm launching with insufficient balance of base tokens on signer's account");
                },
                (_error) => {}
            ));
        })

        it("Launching CpAmm with insufficient balance of quote tokens on signer's account should fail", async () => {
            const cpAmmAccountBefore = await fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(23437123213686);
            const quoteLiquidity = BigInt(1_000_000_000_000_001);

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault1[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault1[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault1[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken1.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken1[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken221.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await (pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm launching with insufficient balance of quote tokens on signer's account");
                },
                (_error) => {}
            ));
        })

        it("Launching CpAmm with signer that isn't CpAmm creator should fail", async () => {
            const cpAmmAccountBefore = await fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(212342403);
            const quoteLiquidity = BigInt(453247832);

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault1[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault1[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault1[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: generalUser,
                signerBaseAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken1.address,
                signerLpAccount:  GENERAL_USER_TOKEN_ACCOUNTS.lpToken1[0],
                signerQuoteAccount:  GENERAL_USER_TOKEN_ACCOUNTS.validToken221.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await (pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm launching with signer that isn't CpAmm creator");
                },
                (_error) => {}
            ));
        })

        it("Launch CpAmm with token mint and token 2022 mint", async () => {
            const [cpAmmAccountBefore, signerBaseBalanceBefore, signerQuoteBalanceBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0]),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken1.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken221.address).send()
            ]);
            const [baseMint, quoteMint, lpMintAccountBefore] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint22(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.lpMint)
            ]);

            const baseLiquidity = BigInt(212342403);
            const quoteLiquidity = BigInt(453247832);
            const totalLiquidity = BigInt(Math.floor(Math.sqrt(Number(baseLiquidity * quoteLiquidity))));

            const initialLockedLiquidity = BigInt(Math.pow(10, lpMintAccountBefore.data.decimals));
            const signersLiquidity = totalLiquidity - initialLockedLiquidity;

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault1[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault1[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault1[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken1.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken1[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken221.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const cpAmmAccountAfter = await fetchCpAmm(rpcClient.rpc, cpAmmAccountBefore.address);

            const [lpMintAccountAfter, signerBaseBalanceAfter, signerQuoteBalanceAfter, signerLpBalanceAfter, cpAmmBaseBalance, cpAmmQuoteBalance, cpAmmLpBalance] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountAfter.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken1.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken221.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.lpToken1[0]).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.lockedLpVault).send()
            ]);

            assert.strictEqual(BigInt(signerBaseBalanceBefore.value.amount) - BigInt(signerBaseBalanceAfter.value.amount), baseLiquidity, "Signer base balance does not match expected value");
            assert.strictEqual(BigInt(signerQuoteBalanceBefore.value.amount) - BigInt(signerQuoteBalanceAfter.value.amount), quoteLiquidity, "Signer quote balance does not match expected value");
            assert.strictEqual(BigInt(signerLpBalanceAfter.value.amount), signersLiquidity, "Signer lp balance does not match expected value");

            assert.strictEqual(BigInt(cpAmmBaseBalance.value.amount), baseLiquidity, "CpAmm base balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmQuoteBalance.value.amount), quoteLiquidity, "CpAmm quote balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmLpBalance.value.amount), initialLockedLiquidity, "CpAmm locked lp balance does not match expected value");

            assert.strictEqual(lpMintAccountAfter.data.supply, totalLiquidity, "LP mint supply is incorrect");

            assert.strictEqual(cpAmmAccountBefore.data.creator, cpAmmAccountAfter.data.creator,  "Creator address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.ammsConfig, cpAmmAccountAfter.data.ammsConfig,  "AMMs config address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.baseMint, cpAmmAccountAfter.data.baseMint, "Base mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.quoteMint, cpAmmAccountAfter.data.quoteMint, "Quote mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.lpMint, cpAmmAccountAfter.data.lpMint, "LP mint address should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.baseVault, TEST_CP_AMMS.baseVault1[0], "Base vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.quoteVault, TEST_CP_AMMS.quoteVault1[0], "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.lockedLpVault, TEST_CP_AMMS.lpVault1[0], "LP vault address mismatch");

            assert.strictEqual(cpAmmAccountBefore.data.protocolBaseFeesToRedeem, cpAmmAccountAfter.data.protocolBaseFeesToRedeem, "Protocol base fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.protocolQuoteFeesToRedeem, cpAmmAccountAfter.data.protocolQuoteFeesToRedeem, "Protocol quote fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.bump[0], cpAmmAccountAfter.data.bump[0], "Bump value should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccountAfter.data.isLaunched, true,  "CpAmm should be launched");

            assert.strictEqual(cpAmmAccountAfter.data.initialLockedLiquidity, initialLockedLiquidity, `Initial locked liquidity should be ${initialLockedLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.lpTokensSupply, totalLiquidity, `LP token supply should be ${totalLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.baseLiquidity, baseLiquidity, `Base liquidity should be ${baseLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.quoteLiquidity, quoteLiquidity, `Quote liquidity should be ${quoteLiquidity}`);

            assert.deepStrictEqual(cpAmmAccountAfter.data.baseQuoteRatioSqrt, {value: [[ 11569318178613274784n, 12626128898751551786n, 0n ]]}, "Base quote ratio sqrt mismatch");
            assert.deepStrictEqual(cpAmmAccountAfter.data.constantProductSqrt, {value: [[ 11035359224094822028n, 1696597754053898133n, 310231742n ]]}, "Constant product sqrt mismatch");
        })

        it("Relaunching of CpAmm should fail", async () => {
            const cpAmmAccountBefore = await fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm1[0]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(212342403);
            const quoteLiquidity = BigInt(453247832);

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault1[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault1[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault1[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken1.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken1[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken221.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await (pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm relaunching");
                },
                (_error) => {}
            ));
        })

        it("Launching CpAmm with launch liquidity less then initial locked liquidity x4 should fail", async () => {
            const [cpAmmAccountBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm2[0])
            ]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(160000);
            const quoteLiquidity = BigInt(999999);

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault2[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault2[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault2[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken2[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken3.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await (pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of CpAmm launching with launch liquidity less then initial locked liquidity x4");
                },
                (_error) => {}
            ));
        })

        it("Launch CpAmm with two token mints", async () => {
            const [cpAmmAccountBefore, signerBaseBalanceBefore, signerQuoteBalanceBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm2[0]),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken3.address).send()
            ]);
            const [baseMint, quoteMint, lpMintAccountBefore] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint22(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.lpMint)
            ]);

            const baseLiquidity = BigInt(160000);
            const quoteLiquidity = BigInt(1_000_000);
            const totalLiquidity = BigInt(Math.floor(Math.sqrt(Number(baseLiquidity * quoteLiquidity))));

            const initialLockedLiquidity = BigInt(Math.pow(10, lpMintAccountBefore.data.decimals));
            const signersLiquidity = totalLiquidity - initialLockedLiquidity;

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault2[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault2[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault2[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken2[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.validToken3.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const cpAmmAccountAfter = await fetchCpAmm(rpcClient.rpc, cpAmmAccountBefore.address);

            const [lpMintAccountAfter, signerBaseBalanceAfter, signerQuoteBalanceAfter, signerLpBalanceAfter, cpAmmBaseBalance, cpAmmQuoteBalance, cpAmmLpBalance] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountAfter.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken3.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.lpToken2[0]).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.lockedLpVault).send()
            ]);

            assert.strictEqual(BigInt(signerBaseBalanceBefore.value.amount) - BigInt(signerBaseBalanceAfter.value.amount), baseLiquidity, "Signer base balance does not match expected value");
            assert.strictEqual(BigInt(signerQuoteBalanceBefore.value.amount) - BigInt(signerQuoteBalanceAfter.value.amount), quoteLiquidity, "Signer quote balance does not match expected value");
            assert.strictEqual(BigInt(signerLpBalanceAfter.value.amount), signersLiquidity, "Signer lp balance does not match expected value");

            assert.strictEqual(BigInt(cpAmmBaseBalance.value.amount), baseLiquidity, "CpAmm base balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmQuoteBalance.value.amount), quoteLiquidity, "CpAmm quote balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmLpBalance.value.amount), initialLockedLiquidity, "CpAmm locked lp balance does not match expected value");

            assert.strictEqual(lpMintAccountAfter.data.supply, totalLiquidity, "LP mint supply is incorrect");

            assert.strictEqual(cpAmmAccountBefore.data.creator, cpAmmAccountAfter.data.creator,  "Creator address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.ammsConfig, cpAmmAccountAfter.data.ammsConfig,  "AMMs config address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.baseMint, cpAmmAccountAfter.data.baseMint, "Base mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.quoteMint, cpAmmAccountAfter.data.quoteMint, "Quote mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.lpMint, cpAmmAccountAfter.data.lpMint, "LP mint address should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.baseVault, TEST_CP_AMMS.baseVault2[0], "Base vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.quoteVault, TEST_CP_AMMS.quoteVault2[0], "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.lockedLpVault, TEST_CP_AMMS.lpVault2[0], "LP vault address mismatch");

            assert.strictEqual(cpAmmAccountBefore.data.protocolBaseFeesToRedeem, cpAmmAccountAfter.data.protocolBaseFeesToRedeem, "Protocol base fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.protocolQuoteFeesToRedeem, cpAmmAccountAfter.data.protocolQuoteFeesToRedeem, "Protocol quote fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.bump[0], cpAmmAccountAfter.data.bump[0], "Bump value should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccountAfter.data.isLaunched, true,  "CpAmm should be launched");

            assert.strictEqual(cpAmmAccountAfter.data.initialLockedLiquidity, initialLockedLiquidity, `Initial locked liquidity should be ${initialLockedLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.lpTokensSupply, totalLiquidity, `LP token supply should be ${totalLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.baseLiquidity, baseLiquidity, `Base liquidity should be ${baseLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.quoteLiquidity, quoteLiquidity, `Quote liquidity should be ${quoteLiquidity}`);

            assert.deepStrictEqual(cpAmmAccountAfter.data.baseQuoteRatioSqrt, {value: [[ 7378697629483820645n, 7378697629483820646n, 0n ] ]}, "Base quote ratio sqrt mismatch");
            assert.deepStrictEqual(cpAmmAccountAfter.data.constantProductSqrt, {value: [[ 0n, 0n, 400000n ]]}, "Constant product sqrt mismatch");
        })

        it("Launch CpAmm with token mint and token 2022 mint with TransferFee Config extension", async () => {
            const [cpAmmAccountBefore, signerBaseBalanceBefore, signerQuoteBalanceBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm3[0]),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.transferFeeToken22.address).send()
            ]);
            const [baseMint, quoteMint, lpMintAccountBefore] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint22(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.lpMint)
            ]);

            const baseLiquidity = BigInt(5465487548754);
            const quoteLiquidity = BigInt(983129578946);

            const transferFee = (quoteMint.data.extensions as Some<Extension[]>).value.find((extension) => extension.__kind == "TransferFeeConfig").olderTransferFee;
            const quoteFee = (quoteLiquidity * BigInt(transferFee.transferFeeBasisPoints) / BigInt(10_000)) < BigInt(transferFee.maximumFee)
                ? (quoteLiquidity * BigInt(transferFee.transferFeeBasisPoints) / BigInt(10_000))
                : BigInt(transferFee.maximumFee);

            const totalLiquidity = BigInt(Math.floor(Math.sqrt(Number(baseLiquidity * (quoteLiquidity - quoteFee)))));

            const initialLockedLiquidity = BigInt(Math.pow(10, lpMintAccountBefore.data.decimals));
            const signersLiquidity = totalLiquidity - initialLockedLiquidity;

            const input: LaunchCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseLiquidity,
                baseMint: cpAmmAccountBefore.data.baseMint,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: TEST_CP_AMMS.baseVault3[0],
                cpAmmLockedLpVault: TEST_CP_AMMS.lpVault3[0],
                cpAmmQuoteVault: TEST_CP_AMMS.quoteVault3[0],
                lpMint: cpAmmAccountBefore.data.lpMint,
                quoteLiquidity,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                signer: user,
                signerBaseAccount: USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount:  USER_TOKEN_ACCOUNTS.lpToken3[0],
                signerQuoteAccount:  USER_TOKEN_ACCOUNTS.transferFeeToken22.address,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
            }

            const ix = getLaunchCpAmmInstruction(input);

            await pipe(
                await createTransactionWithComputeUnits(rpcClient, owner, [ix], 270_000),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const cpAmmAccountAfter = await fetchCpAmm(rpcClient.rpc, cpAmmAccountBefore.address);

            const [lpMintAccountAfter, signerBaseBalanceAfter, signerQuoteBalanceAfter, signerLpBalanceAfter, cpAmmBaseBalance, cpAmmQuoteBalance, cpAmmLpBalance] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountAfter.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.transferFeeToken22.address).send(),
                rpcClient.rpc.getTokenAccountBalance(USER_TOKEN_ACCOUNTS.lpToken3[0]).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.lockedLpVault).send()
            ]);

            assert.strictEqual(BigInt(signerBaseBalanceBefore.value.amount) - BigInt(signerBaseBalanceAfter.value.amount), baseLiquidity, "Signer base balance does not match expected value");
            assert.strictEqual(BigInt(signerQuoteBalanceBefore.value.amount) - BigInt(signerQuoteBalanceAfter.value.amount), quoteLiquidity, "Signer quote balance does not match expected value");
            assert.strictEqual(BigInt(signerLpBalanceAfter.value.amount), signersLiquidity, "Signer lp balance does not match expected value");

            assert.strictEqual(BigInt(cpAmmBaseBalance.value.amount), baseLiquidity, "CpAmm base balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmQuoteBalance.value.amount), quoteLiquidity - quoteFee, "CpAmm quote balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmLpBalance.value.amount), initialLockedLiquidity, "CpAmm locked lp balance does not match expected value");

            assert.strictEqual(lpMintAccountAfter.data.supply, totalLiquidity, "LP mint supply is incorrect");

            assert.strictEqual(cpAmmAccountBefore.data.creator, cpAmmAccountAfter.data.creator,  "Creator address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.ammsConfig, cpAmmAccountAfter.data.ammsConfig,  "AMMs config address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.baseMint, cpAmmAccountAfter.data.baseMint, "Base mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.quoteMint, cpAmmAccountAfter.data.quoteMint, "Quote mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.lpMint, cpAmmAccountAfter.data.lpMint, "LP mint address should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.baseVault, TEST_CP_AMMS.baseVault3[0], "Base vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.quoteVault, TEST_CP_AMMS.quoteVault3[0], "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccountAfter.data.lockedLpVault, TEST_CP_AMMS.lpVault3[0], "LP vault address mismatch");

            assert.strictEqual(cpAmmAccountBefore.data.protocolBaseFeesToRedeem, cpAmmAccountAfter.data.protocolBaseFeesToRedeem, "Protocol base fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.protocolQuoteFeesToRedeem, cpAmmAccountAfter.data.protocolQuoteFeesToRedeem, "Protocol quote fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.bump[0], cpAmmAccountAfter.data.bump[0], "Bump value should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccountAfter.data.isLaunched, true,  "CpAmm should be launched");

            assert.strictEqual(cpAmmAccountAfter.data.initialLockedLiquidity, initialLockedLiquidity, `Initial locked liquidity should be ${initialLockedLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.lpTokensSupply, totalLiquidity, `LP token supply should be ${totalLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.baseLiquidity, baseLiquidity, `Base liquidity should be ${baseLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.quoteLiquidity, quoteLiquidity - quoteFee, `Quote liquidity should be ${quoteLiquidity - quoteFee}`);

            assert.deepStrictEqual(cpAmmAccountAfter.data.baseQuoteRatioSqrt, {value: [[ 3475547461318636948n, 6600456554340055308n, 2n ]]}, "Base quote ratio sqrt mismatch");
            assert.deepStrictEqual(cpAmmAccountAfter.data.constantProductSqrt, {value: [[ 16463856578203948456n, 17179385210221578158n, 2318034170991n ]]}, "Constant product sqrt mismatch");

        })


        // Provide CpAmm

        it("Providing liquidity to CpAmm with invalid token ratio should fail", async () => {
            const cpAmmAccountBefore = await  fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm2[0]);
            const [baseMint, quoteMint] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint)
            ]);

            const baseLiquidity = BigInt(480000);
            const quoteLiquidity = BigInt(3_000_001);

            const input: ProvideToCpAmmInput = {
                baseMint: cpAmmAccountBefore.data.baseMint,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                lpMint: cpAmmAccountBefore.data.lpMint,
                ammsConfig: cpAmmAccountBefore.data.ammsConfig,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: cpAmmAccountBefore.data.baseVault,
                cpAmmQuoteVault: cpAmmAccountBefore.data.quoteVault,
                signer: generalUser,
                signerBaseAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount: GENERAL_USER_TOKEN_ACCOUNTS.lpToken2[0],
                signerQuoteAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken3.address,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
                baseLiquidity,
                quoteLiquidity,
            }

            const ix = getProvideToCpAmmInstruction(input);

            await (pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(
                async (signature) => {
                    console.log(await getTransactionLogs(rpcClient, signature));
                    assert.fail("Expected failure of providing liquidity to CpAmm with invalid token ratio");
                },
                (_error) => {}
            ));
        });

        it("Provide liquidity to CpAmm with two token mints", async () => {
            const [cpAmmAccountBefore, signerBaseBalanceBefore, signerQuoteBalanceBefore, signerLpBalanceBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm2[0]),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken3.address).send(),
                0,
            ]);
            const [baseMint, quoteMint, lpMintAccountBefore, cpAmmBaseBalanceBefore, cpAmmQuoteBalanceBefore, cpAmmLpBalanceBefore] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.lockedLpVault).send()
            ]);

            const baseLiquidity = BigInt(480000);
            const quoteLiquidity = BigInt(3_000_000);
            const providedLiquidity = BigInt(1200000);

            const input: ProvideToCpAmmInput = {
                baseMint: cpAmmAccountBefore.data.baseMint,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                lpMint: cpAmmAccountBefore.data.lpMint,
                ammsConfig: cpAmmAccountBefore.data.ammsConfig,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: cpAmmAccountBefore.data.baseVault,
                cpAmmQuoteVault: cpAmmAccountBefore.data.quoteVault,
                signer: generalUser,
                signerBaseAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount: GENERAL_USER_TOKEN_ACCOUNTS.lpToken2[0],
                signerQuoteAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken3.address,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
                baseLiquidity,
                quoteLiquidity,
            }

            const ix = getProvideToCpAmmInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const cpAmmAccountAfter = await fetchCpAmm(rpcClient.rpc, cpAmmAccountBefore.address);

            const [lpMintAccountAfter, signerBaseBalanceAfter, signerQuoteBalanceAfter, signerLpBalanceAfter, cpAmmBaseBalanceAfter, cpAmmQuoteBalanceAfter, cpAmmLpBalanceAfter] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountAfter.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken3.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.lpToken2[0]).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.lockedLpVault).send()
            ]);

            assert.strictEqual(BigInt(signerBaseBalanceBefore.value.amount) - BigInt(signerBaseBalanceAfter.value.amount), baseLiquidity, "Signer base balance does not match expected value");
            assert.strictEqual(BigInt(signerQuoteBalanceBefore.value.amount) - BigInt(signerQuoteBalanceAfter.value.amount), quoteLiquidity, "Signer quote balance does not match expected value");
            assert.strictEqual(BigInt(signerLpBalanceAfter.value.amount) - BigInt(signerLpBalanceBefore), providedLiquidity, "Signer lp balance does not match expected value");

            assert.strictEqual(BigInt(cpAmmBaseBalanceAfter.value.amount) - BigInt(cpAmmBaseBalanceBefore.value.amount), baseLiquidity, "CpAmm base balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmQuoteBalanceAfter.value.amount) - BigInt(cpAmmQuoteBalanceBefore.value.amount), quoteLiquidity, "CpAmm quote balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmLpBalanceAfter.value.amount), BigInt(cpAmmLpBalanceBefore.value.amount), "CpAmm locked lp balance should remain unchanged");

            assert.strictEqual(lpMintAccountAfter.data.supply - lpMintAccountBefore.data.supply, providedLiquidity, "LP mint supply is incorrect");

            assert.strictEqual(cpAmmAccountBefore.data.ammsConfig, cpAmmAccountAfter.data.ammsConfig,  "AMMs config address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.baseMint, cpAmmAccountAfter.data.baseMint, "Base mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.quoteMint, cpAmmAccountAfter.data.quoteMint, "Quote mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.lpMint, cpAmmAccountAfter.data.lpMint, "LP mint address should remain unchanged");

            assert.strictEqual(cpAmmAccountBefore.data.baseVault, cpAmmAccountAfter.data.baseVault, "Base vault address mismatch");
            assert.strictEqual(cpAmmAccountBefore.data.quoteVault, cpAmmAccountAfter.data.quoteVault, "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccountBefore.data.lockedLpVault, cpAmmAccountAfter.data.lockedLpVault, "LP vault address mismatch");

            assert.strictEqual(cpAmmAccountBefore.data.protocolBaseFeesToRedeem, cpAmmAccountAfter.data.protocolBaseFeesToRedeem, "Protocol base fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.protocolQuoteFeesToRedeem, cpAmmAccountAfter.data.protocolQuoteFeesToRedeem, "Protocol quote fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.bump[0], cpAmmAccountAfter.data.bump[0], "Bump value should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccountAfter.data.isLaunched, true,  "CpAmm should be launched");

            assert.strictEqual(cpAmmAccountBefore.data.initialLockedLiquidity, cpAmmAccountAfter.data.initialLockedLiquidity, `Initial locked liquidity should remain unchanged`);
            assert.strictEqual(cpAmmAccountAfter.data.lpTokensSupply - cpAmmAccountBefore.data.lpTokensSupply, providedLiquidity, `LP token supply should be ${providedLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.baseLiquidity - cpAmmAccountBefore.data.baseLiquidity, baseLiquidity, `Base liquidity should be ${baseLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.quoteLiquidity - cpAmmAccountBefore.data.quoteLiquidity, quoteLiquidity, `Quote liquidity should be ${quoteLiquidity}`);

            assert.deepStrictEqual(cpAmmAccountBefore.data.baseQuoteRatioSqrt, cpAmmAccountAfter.data.baseQuoteRatioSqrt, "Base quote ratio should remain unchanged");
            assert.deepStrictEqual(cpAmmAccountAfter.data.constantProductSqrt, { value: [ [ 0n, 0n, 1600000n ] ] }, "Constant product sqrt mismatch");

        })

        it("Provide liquidity to CpAmm with token mint and token 2022 mint with TransferFee Config extension", async () => {
            const [cpAmmAccountBefore, signerBaseBalanceBefore, signerQuoteBalanceBefore, signerLpBalanceBefore] = await Promise.all([
                fetchCpAmm(rpcClient.rpc, TEST_CP_AMMS.cpAmm3[0]),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.transferFeeToken22.address).send(),
                0,
            ]);
            const [baseMint, quoteMint, lpMintAccountBefore, cpAmmBaseBalanceBefore, cpAmmQuoteBalanceBefore, cpAmmLpBalanceBefore] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.baseMint),
                fetchMint22(rpcClient.rpc, cpAmmAccountBefore.data.quoteMint),
                fetchMint(rpcClient.rpc, cpAmmAccountBefore.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountBefore.data.lockedLpVault).send()
            ]);

            const baseLiquidity = BigInt(5465487548754 * 5);
            const quoteLiquidity = BigInt(983129568946 * 5) + 10000n;

            const transferFee = (quoteMint.data.extensions as Some<Extension[]>).value.find((extension) => extension.__kind == "TransferFeeConfig").olderTransferFee;
            const quoteFee = (quoteLiquidity * BigInt(transferFee.transferFeeBasisPoints) / BigInt(10_000)) < BigInt(transferFee.maximumFee)
                ? (quoteLiquidity * BigInt(transferFee.transferFeeBasisPoints) / BigInt(10_000))
                : BigInt(transferFee.maximumFee);

            const quoteAfterFeeLiquidity = quoteLiquidity - quoteFee;
            const providedLiquidity = BigInt(11590170854955);

            const input: ProvideToCpAmmInput = {
                baseMint: cpAmmAccountBefore.data.baseMint,
                quoteMint: cpAmmAccountBefore.data.quoteMint,
                lpMint: cpAmmAccountBefore.data.lpMint,
                ammsConfig: cpAmmAccountBefore.data.ammsConfig,
                cpAmm: cpAmmAccountBefore.address,
                cpAmmBaseVault: cpAmmAccountBefore.data.baseVault,
                cpAmmQuoteVault: cpAmmAccountBefore.data.quoteVault,
                signer: generalUser,
                signerBaseAccount: GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address,
                signerLpAccount: GENERAL_USER_TOKEN_ACCOUNTS.lpToken3[0],
                signerQuoteAccount: GENERAL_USER_TOKEN_ACCOUNTS.transferFeeToken22.address,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                baseTokenProgram: baseMint.programAddress,
                lpTokenProgram: TOKEN_PROGRAM_ADDRESS,
                quoteTokenProgram: quoteMint.programAddress,
                baseLiquidity,
                quoteLiquidity,
            }

            const ix = getProvideToCpAmmInstruction(input);

            await pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            );

            const cpAmmAccountAfter = await fetchCpAmm(rpcClient.rpc, cpAmmAccountBefore.address);

            const [lpMintAccountAfter, signerBaseBalanceAfter, signerQuoteBalanceAfter, signerLpBalanceAfter, cpAmmBaseBalanceAfter, cpAmmQuoteBalanceAfter, cpAmmLpBalanceAfter] = await Promise.all([
                fetchMint(rpcClient.rpc, cpAmmAccountAfter.data.lpMint),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.validToken2.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.transferFeeToken22.address).send(),
                rpcClient.rpc.getTokenAccountBalance(GENERAL_USER_TOKEN_ACCOUNTS.lpToken3[0]).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.baseVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.quoteVault).send(),
                rpcClient.rpc.getTokenAccountBalance(cpAmmAccountAfter.data.lockedLpVault).send()
            ]);

            assert.strictEqual(BigInt(signerBaseBalanceBefore.value.amount) - BigInt(signerBaseBalanceAfter.value.amount), baseLiquidity, "Signer base balance does not match expected value");
            assert.strictEqual(BigInt(signerQuoteBalanceBefore.value.amount) - BigInt(signerQuoteBalanceAfter.value.amount), quoteLiquidity, "Signer quote balance does not match expected value");
            assert.strictEqual(BigInt(signerLpBalanceAfter.value.amount) - BigInt(signerLpBalanceBefore), providedLiquidity, "Signer lp balance does not match expected value");

            assert.strictEqual(BigInt(cpAmmBaseBalanceAfter.value.amount) - BigInt(cpAmmBaseBalanceBefore.value.amount), baseLiquidity, "CpAmm base balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmQuoteBalanceAfter.value.amount) - BigInt(cpAmmQuoteBalanceBefore.value.amount), quoteAfterFeeLiquidity, "CpAmm quote balance does not match expected value");
            assert.strictEqual(BigInt(cpAmmLpBalanceAfter.value.amount), BigInt(cpAmmLpBalanceBefore.value.amount), "CpAmm locked lp balance should remain unchanged");

            assert.strictEqual(lpMintAccountAfter.data.supply - lpMintAccountBefore.data.supply, providedLiquidity, "LP mint supply is incorrect");

            assert.strictEqual(cpAmmAccountBefore.data.ammsConfig, cpAmmAccountAfter.data.ammsConfig,  "AMMs config address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.baseMint, cpAmmAccountAfter.data.baseMint, "Base mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.quoteMint, cpAmmAccountAfter.data.quoteMint, "Quote mint address should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.lpMint, cpAmmAccountAfter.data.lpMint, "LP mint address should remain unchanged");

            assert.strictEqual(cpAmmAccountBefore.data.baseVault, cpAmmAccountAfter.data.baseVault, "Base vault address mismatch");
            assert.strictEqual(cpAmmAccountBefore.data.quoteVault, cpAmmAccountAfter.data.quoteVault, "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccountBefore.data.lockedLpVault, cpAmmAccountAfter.data.lockedLpVault, "LP vault address mismatch");

            assert.strictEqual(cpAmmAccountBefore.data.protocolBaseFeesToRedeem, cpAmmAccountAfter.data.protocolBaseFeesToRedeem, "Protocol base fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.protocolQuoteFeesToRedeem, cpAmmAccountAfter.data.protocolQuoteFeesToRedeem, "Protocol quote fees should remain unchanged");
            assert.strictEqual(cpAmmAccountBefore.data.bump[0], cpAmmAccountAfter.data.bump[0], "Bump value should remain unchanged");

            assert.strictEqual(cpAmmAccountAfter.data.isInitialized, true,  "CpAmm should be initialized");
            assert.strictEqual(cpAmmAccountAfter.data.isLaunched, true,  "CpAmm should be launched");

            assert.strictEqual(cpAmmAccountBefore.data.initialLockedLiquidity, cpAmmAccountAfter.data.initialLockedLiquidity, `Initial locked liquidity should remain unchanged`);
            assert.strictEqual(cpAmmAccountAfter.data.lpTokensSupply - cpAmmAccountBefore.data.lpTokensSupply, providedLiquidity, `LP token supply should be ${providedLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.baseLiquidity - cpAmmAccountBefore.data.baseLiquidity, baseLiquidity, `Base liquidity should be ${baseLiquidity}`);
            assert.strictEqual(cpAmmAccountAfter.data.quoteLiquidity - cpAmmAccountBefore.data.quoteLiquidity, quoteAfterFeeLiquidity, `Quote liquidity should be ${quoteAfterFeeLiquidity}`);

            assert.deepStrictEqual(cpAmmAccountBefore.data.baseQuoteRatioSqrt, cpAmmAccountAfter.data.baseQuoteRatioSqrt, "Base quote ratio should remain unchanged");
            assert.deepStrictEqual(cpAmmAccountAfter.data.constantProductSqrt, { value: [ [ 6549419100675932660n, 10842590892781710873n, 13908205025951n ]  ] }, "Constant product sqrt mismatch");

        })

        // Swap CpAmm



        // Withdraw CpAmm


    })
}