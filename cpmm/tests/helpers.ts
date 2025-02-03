import {
    address,
    Address,
    airdropFactory,
    appendTransactionMessageInstructions,
    Commitment,
    CompilableTransactionMessage,
    createKeyPairSignerFromBytes,
    createSolanaRpc,
    createSolanaRpcSubscriptions,
    createTransactionMessage,
    Endian,
    generateKeyPairSigner,
    getAddressEncoder,
    getComputeUnitEstimateForTransactionMessageFactory,
    getProgramDerivedAddress,
    getSignatureFromTransaction,
    getU64Encoder,
    IInstruction,
    KeyPairSigner,
    lamports,
    pipe, prependTransactionMessageInstruction,
    ProgramDerivedAddress,
    Rpc,
    RpcSubscriptions,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    SolanaRpcApi,
    SolanaRpcSubscriptionsApi,
    TransactionMessageWithBlockhashLifetime
} from "@solana/web3.js";
import * as program from "../clients/js/src/generated";
import fs from "node:fs";
import {TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda} from "@solana-program/token";
import {TOKEN_2022_PROGRAM_ADDRESS, findAssociatedTokenPda as findAssociatedToken22Pda,} from "@solana-program/token-2022";
import {getSetComputeUnitLimitInstruction} from "@solana-program/compute-budget";

const LAMPORTS_PER_SOL = 1_000_000_000;

export type RpcClient = {
    rpc: Rpc<SolanaRpcApi>;
    rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
};
export const createTestUser = async (rpcClient: RpcClient, airdrop_amount: number): Promise<KeyPairSigner> =>{
    const user = await generateKeyPairSigner();
    console.log("Generated user address:", user.address);

    const airdrop = airdropFactory(rpcClient);
    await airdrop({
        commitment: 'processed',
        lamports: lamports(BigInt(LAMPORTS_PER_SOL * airdrop_amount)),
        recipientAddress: user.address
    });

    console.log(`Airdrop of ${airdrop_amount} SOL completed for user:`, user.address);

    return user;
}


export const createTransaction = async (rpcClient: RpcClient, payer: KeyPairSigner, instructions: IInstruction[]) => {
    const { value: latestBlockhash } = await rpcClient.rpc.getLatestBlockhash().send();
    const transaction = pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(payer, tx),
        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) => appendTransactionMessageInstructions(instructions, tx)
    );
    return transaction;
};

export const createTransactionWithComputeUnits = async (rpcClient: RpcClient, payer: KeyPairSigner, instructions: IInstruction[], computeUnits: number) => {
    const { value: latestBlockhash } = await rpcClient.rpc.getLatestBlockhash().send();
    instructions.unshift(getSetComputeUnitLimitInstruction({ units: computeUnits }))
    const transaction = pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(payer, tx),
        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) => appendTransactionMessageInstructions(instructions, tx)
    );
    return transaction;
};

export const signAndSendTransaction = async (
    rpcClient: RpcClient,
    transactionMessage: CompilableTransactionMessage & TransactionMessageWithBlockhashLifetime,
    commitment: Commitment = 'confirmed'
) => {
    const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);
    await sendAndConfirmTransactionFactory(rpcClient)(signedTransaction, {
        commitment,
    });
    const signature = getSignatureFromTransaction(signedTransaction);
    return signature;
};

export type CpmmTestingEnvironment = {
    program: typeof program,
    rpcClient: RpcClient,
    rent: Address,
    owner: KeyPairSigner,
    headAuthority: KeyPairSigner,
    ammsConfigsManagerAuthority: KeyPairSigner,
    user: KeyPairSigner
};

export const createCpmmTestingEnvironment = async (): Promise<CpmmTestingEnvironment> => {
    const httpEndpoint = 'http://127.0.0.1:8899';
    const wsEndpoint = 'ws://127.0.0.1:8900';
    const rpcClient: RpcClient = {rpc: createSolanaRpc(httpEndpoint), rpcSubscriptions: createSolanaRpcSubscriptions(wsEndpoint)}
    const owner = await createKeyPairSignerFromBytes(Buffer.from(JSON.parse(fs.readFileSync("../owner.json", 'utf8'))));
    const headAuthority = await createTestUser(rpcClient, 100);
    const ammsConfigsManagerAuthority = await createTestUser(rpcClient, 100);
    const user = await createTestUser(rpcClient, 100);
    const rent = address("SysvarRent111111111111111111111111111111111");
    return {rpcClient, headAuthority, owner, program, rent, ammsConfigsManagerAuthority, user};
};

export const getAmmsConfigsManagerPDA = async (): Promise<ProgramDerivedAddress> =>{
    return await getProgramDerivedAddress({programAddress: program.CPMM_PROGRAM_ADDRESS, seeds: ["amms_configs_manager"]});
}

export const getAmmsConfigPDA = async (id: bigint): Promise<ProgramDerivedAddress> =>{
    return await getProgramDerivedAddress({programAddress: program.CPMM_PROGRAM_ADDRESS, seeds: ["amms_config", getU64Encoder({ endian: Endian.Little }).encode(id)]});
}

export const getCpAmmPDA = async (lpMint: Address): Promise<ProgramDerivedAddress> =>{
   return await getProgramDerivedAddress({programAddress: program.CPMM_PROGRAM_ADDRESS, seeds: ["cp_amm", getAddressEncoder().encode(lpMint)]});
}

export const getTokenPDA = async (mint: Address, owner: Address): Promise<ProgramDerivedAddress> => {
    return await findAssociatedTokenPda({
        mint,
        owner,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
}
export const getToken22PDA = async (mint: Address, owner: Address): Promise<ProgramDerivedAddress> => {
    return await findAssociatedToken22Pda({
        mint,
        owner,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });
}