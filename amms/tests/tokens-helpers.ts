import {Account, Address, generateKeyPairSigner, KeyPairSigner, pipe, ProgramDerivedAddress} from "@solana/web3.js";
import {
    fetchMint as fetchTokenMint,
    fetchToken as fetchTokenAccount,
    getInitializeMint2Instruction as getInitializeTokenMint2Instruction,
    getMintSize as getTokenMintSize, Mint as TokenMint, Token as TokenAccount,
    TOKEN_PROGRAM_ADDRESS, getCreateAssociatedTokenIdempotentInstruction, getMintToInstruction, findAssociatedTokenPda
} from "@solana-program/token";
import {
    fetchMint as fetchToken22Mint,
    fetchToken as fetchToken22Account,
    getInitializeMint2Instruction as getInitializeToken22Mint2Instruction,
    getMintSize as getToken22MintSize,
    TOKEN_2022_PROGRAM_ADDRESS,
    getInitializePermanentDelegateInstruction,
    getInitializeTransferFeeConfigInstruction,
    Mint as Token22Mint,
    Token as Token22Account,
    getCreateAssociatedTokenIdempotentInstruction as getCreateAssociatedToken22IdempotentInstruction,
    getMintToInstruction as getMint22ToInstruction,
    findAssociatedTokenPda as findAssociatedToken22Pda
} from "@solana-program/token-2022";
import {getCreateAccountInstruction} from "@solana-program/system";
import {createTransaction, RpcClient, signAndSendTransaction} from "./helpers";

/**
 * Retrieves the PDA for a standard token account.
 * @param {Address} mint - The token mint address.
 * @param {Address} owner - The owner's address.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address for the token account.
 */
export const getTokenPDA = async (mint: Address, owner: Address): Promise<ProgramDerivedAddress> => {
    return await findAssociatedTokenPda({
        mint,
        owner,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
};

/**
 * Retrieves the PDA for a Token-2022 token account.
 * @param {Address} mint - The token mint address.
 * @param {Address} owner - The owner's address.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address for the Token-2022 account.
 */
export const getToken22PDA = async (mint: Address, owner: Address): Promise<ProgramDerivedAddress> => {
    return await findAssociatedToken22Pda({
        mint,
        owner,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });
};

/**
 * Creates a new token mint and initializes it with specified parameters.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} authority - The authority responsible for managing the mint.
 * @param {number} decimals - The number of decimal places for the token.
 * @param {Address} [freezeAuthority] - Optional freeze authority for the mint.
 * @returns {Promise<Account<TokenMint>>} - The created token mint account.
 */
export const createTokenMint = async (rpcClient: RpcClient, authority: KeyPairSigner, decimals: number, freezeAuthority?: Address): Promise<Account<TokenMint>> => {
    const mint = await generateKeyPairSigner();
    const mintSpace = BigInt(getTokenMintSize());
    const mintRent = await rpcClient.rpc.getMinimumBalanceForRentExemption(mintSpace).send();

    const instructions = [
        getCreateAccountInstruction({
            payer: authority,
            newAccount: mint,
            lamports: mintRent,
            space: mintSpace,
            programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        getInitializeTokenMint2Instruction({
            mint: mint.address,
            decimals,
            mintAuthority: authority.address,
            freezeAuthority
        }),
    ];

    await pipe(
        await createTransaction(rpcClient, authority, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchTokenMint(rpcClient.rpc, mint.address);
}

/**
 * Creates a new Token-2022 mint.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} authority - The mint authority.
 * @param {number} decimals - The number of decimal places for the token.
 * @param {Address} [freezeAuthority] - Optional freeze authority.
 * @returns {Promise<Account<Token22Mint>>} - The created Token-2022 mint account.
 */
export const createToken22Mint = async (rpcClient: RpcClient, authority: KeyPairSigner, decimals: number, freezeAuthority?: Address): Promise<Account<Token22Mint>> => {
    const mint = await generateKeyPairSigner();
    const mintSpace = BigInt(getToken22MintSize());
    const mintRent = await rpcClient.rpc.getMinimumBalanceForRentExemption(mintSpace).send();
    const instructions = [
        getCreateAccountInstruction({
            payer: authority,
            newAccount: mint,
            lamports: mintRent,
            space: mintSpace,
            programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        }),
        getInitializeToken22Mint2Instruction({
            mint: mint.address,
            decimals,
            mintAuthority: authority.address,
            freezeAuthority
        })
    ];

    await pipe(
        await createTransaction(rpcClient, authority, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchToken22Mint(rpcClient.rpc, mint.address);
}

/**
 * Creates a new Token-2022 mint with transfer fee configuration.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} authority - The mint authority.
 * @param {number} decimals - The number of decimal places.
 * @param {number} feeBasisPoints - The transfer fee basis points.
 * @param {number} maximumFee - The maximum transfer fee.
 * @param {Address} [freezeAuthority] - Optional freeze authority.
 * @returns {Promise<Account<Token22Mint>>} - The created Token-2022 mint account.
 */
export const createToken22MintWithTransferFee = async (rpcClient: RpcClient, authority: KeyPairSigner, decimals: number, feeBasisPoints: number, maximumFee: number, freezeAuthority?: Address): Promise<Account<Token22Mint>> => {
    const mint = await generateKeyPairSigner();
    const mintSpace = BigInt(getToken22MintSize([
        {
            __kind: "TransferFeeConfig",
            transferFeeConfigAuthority: authority.address,
            withdrawWithheldAuthority: authority.address,
            withheldAmount: 0,
            olderTransferFee: {
                epoch: 0,
                maximumFee: 0,
                transferFeeBasisPoints: 0
            },
            newerTransferFee: {
                epoch: 0,
                maximumFee: 0,
                transferFeeBasisPoints: 0
            },
        }
    ]));
    const mintRent = await rpcClient.rpc.getMinimumBalanceForRentExemption(mintSpace).send();
    const instructions = [
        getCreateAccountInstruction({
            payer: authority,
            newAccount: mint,
            lamports: mintRent,
            space: mintSpace,
            programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        }),
        getInitializeTransferFeeConfigInstruction({
            maximumFee,
            mint: mint.address,
            transferFeeBasisPoints: feeBasisPoints,
            transferFeeConfigAuthority: authority.address,
            withdrawWithheldAuthority: authority.address
        }),
        getInitializeToken22Mint2Instruction({
            mint: mint.address,
            decimals,
            mintAuthority: authority.address,
            freezeAuthority
        })
    ];

    await pipe(
        await createTransaction(rpcClient, authority, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchToken22Mint(rpcClient.rpc, mint.address);
}

/**
 * Creates a Token-2022 mint with a permanent delegate.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} authority - The mint authority.
 * @param {number} decimals - The number of decimal places for the token.
 * @param {Address} [freezeAuthority] - Optional freeze authority.
 * @returns {Promise<Account<Token22Mint>>} - The created Token-2022 mint account.
 */
export const createToken22MintWithPermanentDelegate = async (rpcClient: RpcClient, authority: KeyPairSigner, decimals: number, freezeAuthority?: Address): Promise<Account<Token22Mint>> => {
    const mint = await generateKeyPairSigner();
    const mintSpace = BigInt(getToken22MintSize([{
        __kind: "PermanentDelegate",
        delegate: authority.address
    }]));
    const mintRent = await rpcClient.rpc.getMinimumBalanceForRentExemption(mintSpace).send();

    const instructions = [
        getCreateAccountInstruction({
            payer: authority,
            newAccount: mint,
            lamports: mintRent,
            space: mintSpace,
            programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        }),
        getInitializePermanentDelegateInstruction({
            delegate: authority.address,
            mint: mint.address
        }),
        getInitializeToken22Mint2Instruction({
            mint: mint.address,
            decimals,
            mintAuthority: authority.address,
            freezeAuthority
        })
    ];

    await pipe(
        await createTransaction(rpcClient, authority, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchToken22Mint(rpcClient.rpc, mint.address);
}

/**
 * Creates an associated token account and mints tokens into it.
 * @param {RpcClient} rpcClient - The Solana RPC client for executing transactions.
 * @param {Address} mint - The token mint address for which the ATA will be created.
 * @param {KeyPairSigner} mintAuthority - The authority responsible for minting tokens.
 * @param {KeyPairSigner} recipient - The recipient of the minted tokens.
 * @param {bigint} mintAmount - The amount of tokens to mint into the ATA.
 * @returns {Promise<Account<TokenAccount>>} - The created token account with minted tokens.
 */
export const createAtaWithTokens = async (rpcClient: RpcClient, mint: Address, mintAuthority: KeyPairSigner, recipient: KeyPairSigner, mintAmount: bigint): Promise<Account<TokenAccount>> => {
    const [ata] = await getTokenPDA(mint, recipient.address);
    const instructions = [
        getCreateAssociatedTokenIdempotentInstruction({
            ata,
            mint,
            owner: recipient.address,
            payer: recipient
        }),
        getMintToInstruction({
            mint,
            token: ata,
            amount: mintAmount,
            mintAuthority
        })
    ];

    await pipe(
        await createTransaction(rpcClient, recipient, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchTokenAccount(rpcClient.rpc, ata);
};

/**
 * Creates an associated token account for Token-2022 and mints tokens into it.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {Address} mint - The Token-2022 mint address.
 * @param {KeyPairSigner} mintAuthority - The authority responsible for minting tokens.
 * @param {KeyPairSigner} recipient - The recipient of the minted tokens.
 * @param {bigint} mintAmount - The amount of tokens to mint into the ATA.
 * @returns {Promise<Account<Token22Account>>} - The created Token-2022 account with minted tokens.
 */
export const createAtaWithTokens22 = async (rpcClient: RpcClient, mint: Address, mintAuthority: KeyPairSigner, recipient: KeyPairSigner, mintAmount: bigint): Promise<Account<Token22Account>> => {
    const [ata] = await getToken22PDA(mint, recipient.address);
    const instructions = [
        getCreateAssociatedToken22IdempotentInstruction({
            ata,
            mint,
            owner: recipient.address,
            payer: recipient
        }),
        getMint22ToInstruction({
            mint,
            token: ata,
            amount: mintAmount,
            mintAuthority
        })
    ];

    await pipe(
        await createTransaction(rpcClient, recipient, instructions),
        (tx) => signAndSendTransaction(rpcClient, tx)
    );
    return await fetchToken22Account(rpcClient.rpc, ata);
};
