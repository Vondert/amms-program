import {Account, Address, generateKeyPairSigner, KeyPairSigner, pipe} from "@solana/web3.js";
import {
    fetchMint as fetchTokenMint,
    fetchToken as fetchTokenAccount,
    getInitializeMint2Instruction as getInitializeTokenMint2Instruction,
    getMintSize as getTokenMintSize, Mint as TokenMint, Token as TokenAccount,
    TOKEN_PROGRAM_ADDRESS, getCreateAssociatedTokenIdempotentInstruction, getMintToInstruction
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
    getMintToInstruction as getMint22ToInstruction
} from "@solana-program/token-2022";
import {getCreateAccountInstruction} from "@solana-program/system";
import {createTransaction, getToken22PDA, getTokenPDA, RpcClient, signAndSendTransaction} from "./helpers";

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

export const createAtaWithTokens = async (rpcClient: RpcClient, mint: Address, mintAuthority: KeyPairSigner, recipient: KeyPairSigner, mintAmount: bigint): Promise<Account<TokenAccount>> =>{
    const [ata] = await getTokenPDA(mint, recipient.address);

    const instructions = [
        getCreateAssociatedTokenIdempotentInstruction({
            ata,
            mint,
            owner: recipient.address,
            payer: recipient
        }),
        getMintToInstruction({
            mint: mint,
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
}

export const createAtaWithTokens22 = async (rpcClient: RpcClient, mint: Address, mintAuthority: KeyPairSigner, recipient: KeyPairSigner, mintAmount: bigint): Promise<Account<Token22Account>> =>{
    const [ata] = await getToken22PDA(mint, recipient.address);

    const instructions = [
        getCreateAssociatedToken22IdempotentInstruction({
            ata,
            mint,
            owner: recipient.address,
            payer: recipient
        }),
        getMint22ToInstruction({
            mint: mint,
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
}