import {
    Account,
    generateKeyPairSigner, getAddressEncoder, getProgramDerivedAddress,
    KeyPairSigner, none,
    pipe,
    ProgramDerivedAddress, some
} from "@solana/web3.js";
import {before, describe} from "mocha";
import {CpmmTestingEnvironment, createTestUser, createTransaction, signAndSendTransaction} from "./helpers";
import {
    createAtaWithTokens, createAtaWithTokens22,
    createToken22Mint,
    createToken22MintWithPermanentDelegate,
    createToken22MintWithTransferFee,
    createTokenMint
} from "./tokens-helpers";
import {
    ASSOCIATED_TOKEN_PROGRAM_ADDRESS, fetchMint, findAssociatedTokenPda,
    Mint as TokenMint,
    Token as TokenAccount,
    TOKEN_PROGRAM_ADDRESS
} from "@solana-program/token";
import {Mint as Token22Mint, Token as Token22Account} from "@solana-program/token-2022";
import {
    fetchCpAmm,
    getInitializeCpAmmInstruction,
    InitializeCpAmmInput
} from "../clients/js/src/generated";
import {SYSTEM_PROGRAM_ADDRESS} from "@solana-program/system";
import {assert} from "chai";

export const cpAmmTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigAddress: ProgramDerivedAddress, lpMint: KeyPairSigner, cpAmm: ProgramDerivedAddress) =>{
    describe("\nCpAmm tests", () =>{
        const {program, rpcClient, rent, headAuthority, owner, user} = cpmmTestingEnvironment;

        let generalUser: KeyPairSigner;
        let userLpAta: ProgramDerivedAddress;

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
        } = {
            validToken1: undefined,
            validToken2: undefined,
            validToken3: undefined,
            validToken221: undefined,
            validToken222: undefined,
            transferFeeToken22: undefined
        };
        const GENERAL_USER_TOKEN_ACCOUNTS: {
            validToken1: Account<TokenAccount>,
            validToken2: Account<TokenAccount>,
            validToken3: Account<TokenAccount>,
            validToken221: Account<Token22Account>,
            validToken222: Account<Token22Account>,
            transferFeeToken22: Account<Token22Account>,
        } = {
            validToken1: undefined,
            validToken2: undefined,
            validToken3: undefined,
            validToken221: undefined,
            validToken222: undefined,
            transferFeeToken22: undefined
        };

        before(async () =>{
            generalUser = await createTestUser(rpcClient, 100);
            userLpAta = await findAssociatedTokenPda({
                mint: lpMint.address,
                owner: user.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });
            TEST_MINTS.validTokenMint1 = await createTokenMint(rpcClient, user, 6);
            TEST_MINTS.validTokenMint2 = await createTokenMint(rpcClient, user, 4);
            TEST_MINTS.validTokenMint3 = await createTokenMint(rpcClient, user, 9);
            TEST_MINTS.freezeAuthorityTokenMint = await createTokenMint(rpcClient, user, 1, user.address);

            TEST_MINTS.validToken22Mint1 = await createToken22Mint(rpcClient, user, 3);
            TEST_MINTS.validToken22Mint2 = await createToken22Mint(rpcClient, user, 0);
            TEST_MINTS.transferFeeToken2022Mint = await createToken22MintWithTransferFee(rpcClient, user, 2, 379, 10000);
            TEST_MINTS.permanentDelegateToken2022Mint = await createToken22MintWithPermanentDelegate(rpcClient, user, 0);

            USER_TOKEN_ACCOUNTS.validToken1 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint1.address, user, user, BigInt(1_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken2 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint2.address, user, user, BigInt(1_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken3 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint3.address, user, user, BigInt(1_000_000_000));

            USER_TOKEN_ACCOUNTS.validToken221 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint1.address, user, user, BigInt(1_000_000_000));
            USER_TOKEN_ACCOUNTS.validToken222 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint2.address, user, user, BigInt(1_000_000_000));
            USER_TOKEN_ACCOUNTS.transferFeeToken22 = await createAtaWithTokens22(rpcClient, TEST_MINTS.transferFeeToken2022Mint.address, user, user, BigInt(1_000_000_000));

            GENERAL_USER_TOKEN_ACCOUNTS.validToken1 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint1.address, user, generalUser, BigInt(1_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken2 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint2.address, user, generalUser, BigInt(1_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken3 = await createAtaWithTokens(rpcClient, TEST_MINTS.validTokenMint3.address, user, generalUser, BigInt(1_000_000_000));

            GENERAL_USER_TOKEN_ACCOUNTS.validToken221 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint1.address, user, generalUser, BigInt(1_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.validToken222 = await createAtaWithTokens22(rpcClient, TEST_MINTS.validToken22Mint2.address, user, generalUser, BigInt(1_000_000_000));
            GENERAL_USER_TOKEN_ACCOUNTS.transferFeeToken22 = await createAtaWithTokens22(rpcClient, TEST_MINTS.transferFeeToken2022Mint.address, user, generalUser, BigInt(1_000_000_000));
        })

        // Initialize CpAmm (Error)

        it("Unfunded with 0.1 SOL CpAmm initialization attempt should fail", async () => {
            let unfundedUser = await createTestUser(rpcClient, 0.1);
            const [ata] = await findAssociatedTokenPda({
                mint: lpMint.address,
                owner: unfundedUser.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: unfundedUser,
                signerLpTokenAccount: ata,
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of unfunded with 0.1 SOL CpAmm initialization attempt")).catch();
        })

        it("Initialization CpAmm with equal mints should fail", async () => {

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization with equal mints")).catch();
        })

        it("Initialization CpAmm with invalid fee authority should fail", async () => {

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: user.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization with invalid fee authority")).catch();
        })

        it("Initialization CpAmm with malware AmmsConfig should fail", async () => {
            const malwareAmmsConfigAddress = TEST_MINTS.validTokenMint2.address;
            let input: InitializeCpAmmInput = {
                ammsConfig: malwareAmmsConfigAddress,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization malware AmmsConfig")).catch();
        })

        it("Initialization CpAmm with invalid LP mint should fail", async () => {
            const invalidLpMint = await generateKeyPairSigner();
            const [ata] = await findAssociatedTokenPda({
                mint: invalidLpMint.address,
                owner: user.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
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

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization with invalid LP mint")).catch();
        })

        it("Initialization CpAmm with mint with freeze authority should fail", async () => {

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.freezeAuthorityTokenMint.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization with mint with freeze authority")).catch();
        })

        it("Initialization CpAmm with mint with one of forbidden token extensions (Permanent Delegate) should fail", async () => {
            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.permanentDelegateToken2022Mint.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validTokenMint2.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm initialization with mint with one of forbidden token extensions (Permanent Delegate)")).catch();
        })

        // Initialize CpAmm

        it("Initialization CpAmm with token mint and token 2022", async () => {
            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validToken22Mint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
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
                fetchMint(rpcClient.rpc, lpMint.address),
                fetchCpAmm(rpcClient.rpc, cpAmm[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint1.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.validToken22Mint1.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "Fee authority does not match expected value");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "Fee authority does not match expected value");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], cpAmm[1].valueOf(), "Bump value is incorrect");
        })

        it("Reinitialization of CpAmm should fail", async () => {
            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint1.address,
                cpAmm: cpAmm[0],
                feeAuthority: headAuthority.address,
                lpMint,
                quoteMint: TEST_MINTS.validToken22Mint1.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta[0],
                systemProgram: SYSTEM_PROGRAM_ADDRESS,
                tokenProgram: TOKEN_PROGRAM_ADDRESS
            }

            const ix = getInitializeCpAmmInstruction(input);

            pipe(
                await createTransaction(rpcClient, owner, [ix]),
                (tx) => signAndSendTransaction(rpcClient, tx)
            ).then(() => assert.fail("Expected failure of CpAmm reinitialization")).catch();
        })

        it("Initialization CpAmm with two token mints", async () => {
            const lpMint2 = await generateKeyPairSigner();
            const cpAmm2 = await getProgramDerivedAddress({programAddress: program.CPMM_PROGRAM_ADDRESS, seeds: ["cp_amm", getAddressEncoder().encode(lpMint2.address)]});
            const userLpAta2 = await findAssociatedTokenPda({
                mint: lpMint2.address,
                owner: user.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });

            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint2.address,
                cpAmm: cpAmm2[0],
                feeAuthority: headAuthority.address,
                lpMint: lpMint2,
                quoteMint: TEST_MINTS.validTokenMint3.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta2[0],
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
                fetchMint(rpcClient.rpc, lpMint2.address),
                fetchCpAmm(rpcClient.rpc, cpAmm2[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint2.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.validTokenMint3.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "Fee authority does not match expected value");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "Fee authority does not match expected value");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], cpAmm2[1].valueOf(), "Bump value is incorrect");
        })

        it("Initialization CpAmm with token mint and token 2022 with one allowed extension (Transfer Fee Config)", async () => {
            const lpMint3 = await generateKeyPairSigner();
            const cpAmm3 = await getProgramDerivedAddress({programAddress: program.CPMM_PROGRAM_ADDRESS, seeds: ["cp_amm", getAddressEncoder().encode(lpMint3.address)]});
            const userLpAta3 = await findAssociatedTokenPda({
                mint: lpMint3.address,
                owner: user.address,
                tokenProgram: TOKEN_PROGRAM_ADDRESS,
            });

            const feeAuthorityBalanceBefore = await rpcClient.rpc.getBalance(headAuthority.address).send();

            let input: InitializeCpAmmInput = {
                ammsConfig: ammsConfigAddress[0],
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                baseMint: TEST_MINTS.validTokenMint2.address,
                cpAmm: cpAmm3[0],
                feeAuthority: headAuthority.address,
                lpMint: lpMint3,
                quoteMint: TEST_MINTS.transferFeeToken2022Mint.address,
                rent,
                signer: user,
                signerLpTokenAccount: userLpAta3[0],
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
                fetchMint(rpcClient.rpc, lpMint3.address),
                fetchCpAmm(rpcClient.rpc, cpAmm3[0])
            ]);

            assert.ok(lpMintAccount, "LP mint account was not created");
            assert.ok(cpAmmAccount, "CpAmm account was not created");

            assert.strictEqual((feeAuthorityBalanceAfter.value - feeAuthorityBalanceBefore.value), BigInt(100_000_000), "Fee authority balance does not match expected value");

            assert.deepStrictEqual(lpMintAccount.data.mintAuthority, some(cpAmmAccount.address), "LP mint authority is incorrect");
            assert.deepStrictEqual(lpMintAccount.data.freezeAuthority, none(), "LP mint freeze authority should be none");

            assert.strictEqual(cpAmmAccount.data.ammsConfig, ammsConfigAddress[0],  "AMMs config address mismatch");
            assert.strictEqual(cpAmmAccount.data.baseMint, TEST_MINTS.validTokenMint2.address, "Base mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteMint, TEST_MINTS.transferFeeToken2022Mint.address, "Quote mint address mismatch");
            assert.strictEqual(cpAmmAccount.data.lpMint, lpMintAccount.address, "LP mint address mismatch");

            assert.strictEqual(cpAmmAccount.data.baseVault.toString(), "11111111111111111111111111111111", "Base vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.quoteVault.toString(), "11111111111111111111111111111111", "Quote vault address mismatch");
            assert.strictEqual(cpAmmAccount.data.lockedLpVault.toString(), "11111111111111111111111111111111", "LP vault address mismatch");

            assert.strictEqual(cpAmmAccount.data.isInitialized, true,  "Fee authority does not match expected value");
            assert.strictEqual(cpAmmAccount.data.isLaunched, false,  "Fee authority does not match expected value");

            assert.strictEqual(cpAmmAccount.data.initialLockedLiquidity, BigInt(0), "Initial locked liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.lpTokensSupply, BigInt(0), "LP token supply should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolBaseFeesToRedeem, BigInt(0), "Protocol base fees should be 0");
            assert.strictEqual(cpAmmAccount.data.protocolQuoteFeesToRedeem, BigInt(0), "Protocol quote fees should be 0");
            assert.strictEqual(cpAmmAccount.data.baseLiquidity, BigInt(0), "Base liquidity should be 0");
            assert.strictEqual(cpAmmAccount.data.quoteLiquidity, BigInt(0), "Quote liquidity should be 0");

            assert.deepStrictEqual(cpAmmAccount.data.baseQuoteRatioSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Base quote ratio sqrt should be initialized to 0");
            assert.deepStrictEqual(cpAmmAccount.data.constantProductSqrt, {value: [[BigInt(0), BigInt(0), BigInt(0)]]}, "Constant product sqrt should be initialized to 0");

            assert.strictEqual(cpAmmAccount.data.bump[0], cpAmm3[1].valueOf(), "Bump value is incorrect");
        })
    })
}