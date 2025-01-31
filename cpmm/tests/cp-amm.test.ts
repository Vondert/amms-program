import {Account, KeyPairSigner, ProgramDerivedAddress} from "@solana/web3.js";
import {before, describe} from "mocha";
import {CpmmTestingEnvironment, createTestUser} from "./helpers";
import {
    createAtaWithTokens, createAtaWithTokens22,
    createToken22Mint,
    createToken22MintWithPermanentDelegate,
    createToken22MintWithTransferFee,
    createTokenMint
} from "./tokens-helpers";
import {Mint as TokenMint, Token as TokenAccount} from "@solana-program/token";
import {Mint as Token22Mint, Token as Token22Account} from "@solana-program/token-2022";

export const cpAmmTests = (cpmmTestingEnvironment: CpmmTestingEnvironment, ammsConfigAddress: ProgramDerivedAddress, lpMint: KeyPairSigner, cpAmm: ProgramDerivedAddress) =>{
    describe("\nCpAmm tests", () =>{
        const {program, rpcClient, rent, headAuthority, owner, ammsConfigsManagerAuthority, user} = cpmmTestingEnvironment;

        let generalUser: KeyPairSigner;
        const TEST_MINTS: {
            validTokenMint1: Account<TokenMint>,
            validTokenMint2: Account<TokenMint>,
            validTokenMint3: Account<TokenMint>,
            invalidTokenMint: Account<TokenMint>,
            validToken22Mint1: Account<Token22Mint>,
            validToken22Mint2: Account<Token22Mint>,
            transferFeeToken2022Mint: Account<Token22Mint>,
            permanentDelegateToken2022Mint: Account<Token22Mint>
        } = {
            validTokenMint1: undefined,
            validTokenMint2: undefined,
            validTokenMint3: undefined,
            invalidTokenMint: undefined,
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

            TEST_MINTS.validTokenMint1 = await createTokenMint(rpcClient, user, 6);
            TEST_MINTS.validTokenMint2 = await createTokenMint(rpcClient, user, 4);
            TEST_MINTS.validTokenMint3 = await createTokenMint(rpcClient, user, 9);
            TEST_MINTS.invalidTokenMint = await createTokenMint(rpcClient, user, 1, user.address);

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
        it("Initialize CpAmm", async () => {

        })
    })
}