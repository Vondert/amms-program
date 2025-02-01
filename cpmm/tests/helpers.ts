import {
    address,
    Address,
    airdropFactory, appendTransactionMessageInstructions,
    Commitment,
    CompilableTransactionMessage, createKeyPairSignerFromBytes,
    createSolanaRpc,
    createSolanaRpcSubscriptions, createTransactionMessage,
    generateKeyPairSigner,
    getSignatureFromTransaction, IInstruction,
    KeyPairSigner,
    lamports, pipe,
    Rpc,
    RpcSubscriptions,
    sendAndConfirmTransactionFactory, setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    SolanaRpcApi,
    SolanaRpcSubscriptionsApi,
    TransactionMessageWithBlockhashLifetime
} from "@solana/web3.js";
import * as program from "../clients/js/src/generated";
import fs from "node:fs";

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