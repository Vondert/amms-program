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
    getProgramDerivedAddress,
    getSignatureFromTransaction,
    getU64Encoder,
    IInstruction,
    KeyPairSigner,
    lamports,
    pipe,
    ProgramDerivedAddress,
    Rpc,
    RpcSubscriptions,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash, Signature,
    signTransactionMessageWithSigners,
    SolanaRpcApi,
    SolanaRpcSubscriptionsApi,
    TransactionMessageWithBlockhashLifetime,
} from "@solana/web3.js";
import * as program from "../clients/js/src/generated";
import fs from "node:fs";
import {getSetComputeUnitLimitInstruction} from "@solana-program/compute-budget";
import {BaseTransactionMessage, TransactionMessage} from "@solana/transaction-messages/dist/types/transaction-message";

const LAMPORTS_PER_SOL = 1_000_000_000;

/**
 * Defines the RPC client interface with standard Solana API and subscriptions.
 */
export type RpcClient = {
    rpc: Rpc<SolanaRpcApi>;
    rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
};

/**
 * Creates a test user by generating a key pair and airdropping SOL.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {number} airdrop_amount - The amount of SOL to airdrop.
 * @returns {Promise<KeyPairSigner>} - The generated test user's key pair.
 */
export const createTestUser = async (rpcClient: RpcClient, airdrop_amount: number): Promise<KeyPairSigner> => {
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
};

/**
 * Creates a basic transaction with given instructions.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} payer - The transaction fee payer.
 * @param {IInstruction[]} instructions - The instructions to include.
 */
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

/**
 * Creates a transaction with a specified compute unit limit.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {KeyPairSigner} payer - The transaction fee payer.
 * @param {IInstruction[]} instructions - The instructions to include.
 * @param {number} computeUnits - The compute units limit.
 */
export const createTransactionWithComputeUnits = async (rpcClient: RpcClient, payer: KeyPairSigner, instructions: IInstruction[], computeUnits: number) => {
    const { value: latestBlockhash } = await rpcClient.rpc.getLatestBlockhash().send();
    instructions.unshift(getSetComputeUnitLimitInstruction({ units: computeUnits }));
    const transaction = pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(payer, tx),
        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) => appendTransactionMessageInstructions(instructions, tx)
    );
    return transaction;
};

/**
 * Signs and sends a transaction, then returns the transaction signature.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {Commitment} [commitment='confirmed'] - The transaction commitment level.
 * @param {transactionMessage: CompilableTransactionMessage & TransactionMessageWithBlockhashLifetime} - Transaction.
 * @returns {Promise<Signature>} - The transaction signature.
 */
export const signAndSendTransaction = async (
    rpcClient: RpcClient,
    transactionMessage: CompilableTransactionMessage & TransactionMessageWithBlockhashLifetime,
    commitment: Commitment = 'confirmed'
): Promise<Signature> => {
    const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);
    await sendAndConfirmTransactionFactory(rpcClient)(signedTransaction, { commitment });
    return getSignatureFromTransaction(signedTransaction);
};

/**
 * Retrieves logs from a transaction.
 * @param {RpcClient} rpcClient - The Solana RPC client.
 * @param {Signature} signature - The transaction signature.
 * @returns {Promise<readonly string[]>} - The transaction logs.
 */
export const getTransactionLogs = async (rpcClient: RpcClient, signature: Signature): Promise<readonly string[]> => {
    return (await rpcClient.rpc.getTransaction(signature, { maxSupportedTransactionVersion: 0 }).send()).meta?.logMessages;
};

/**
 * Defines the structure for a CPMM testing environment.
 */
export type CpmmTestingEnvironment = {
    program: typeof program,
    programDataAddress: Address,
    rpcClient: RpcClient,
    rent: Address,
    owner: KeyPairSigner,
    headAuthority: KeyPairSigner,
    ammsConfigsManagerAuthority: KeyPairSigner,
    user: KeyPairSigner
};

/**
 * Creates a CPMM testing environment with predefined configurations.
 * @returns {Promise<CpmmTestingEnvironment>} - The initialized testing environment.
 */
export const createCpmmTestingEnvironment = async (): Promise<CpmmTestingEnvironment> => {
    const httpEndpoint = 'http://127.0.0.1:8899';
    const wsEndpoint = 'ws://127.0.0.1:8900';

    // Initialize RPC client for interaction with Solana
    const rpcClient: RpcClient = {
        rpc: createSolanaRpc(httpEndpoint),
        rpcSubscriptions: createSolanaRpcSubscriptions(wsEndpoint)
    };

    // Load owner key pair from a file
    const owner = await createKeyPairSignerFromBytes(Buffer.from(JSON.parse(fs.readFileSync("../owner.json", 'utf8'))));

    // Create test users and authorities
    const headAuthority = await createTestUser(rpcClient, 100);
    const ammsConfigsManagerAuthority = await createTestUser(rpcClient, 100);
    const user = await createTestUser(rpcClient, 100);

    // Define rent system address
    const rent = address("SysvarRent111111111111111111111111111111111");

    // Derive program data address using CPMM program address
    const [programDataAddress] = await getProgramDerivedAddress({
        programAddress: address('BPFLoaderUpgradeab1e11111111111111111111111'),
        seeds: [getAddressEncoder().encode(program.CPMM_PROGRAM_ADDRESS)]
    });

    return { rpcClient, headAuthority, owner, program, rent, programDataAddress, ammsConfigsManagerAuthority, user };
};

/**
 * Retrieves the PDA (Program Derived Address) for AMMs Configs Manager.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address of AMMs Configs Manager.
 */
export const getAmmsConfigsManagerPDA = async (): Promise<ProgramDerivedAddress> => {
    return await getProgramDerivedAddress({
        programAddress: program.CPMM_PROGRAM_ADDRESS,
        seeds: ["amms_configs_manager"]
    });
};

/**
 * Retrieves the PDA for a specific AMMs Config using an ID.
 * @param {bigint} id - The unique identifier for the AMMs Config.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address for the AMMs Config.
 */
export const getAmmsConfigPDA = async (id: bigint): Promise<ProgramDerivedAddress> => {
    return await getProgramDerivedAddress({
        programAddress: program.CPMM_PROGRAM_ADDRESS,
        seeds: ["amms_config", getU64Encoder({ endian: Endian.Little }).encode(id)]
    });
};

/**
 * Retrieves the PDA for a constant product AMM.
 * @param {Address} lpMint - The address of the liquidity pool mint.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address for the constant product AMM.
 */
export const getCpAmmPDA = async (lpMint: Address): Promise<ProgramDerivedAddress> => {
    return await getProgramDerivedAddress({
        programAddress: program.CPMM_PROGRAM_ADDRESS,
        seeds: ["cp_amm", getAddressEncoder().encode(lpMint)]
    });
};

/**
 * Retrieves the PDA for an AMM vault.
 * @param {Address} cpAmm - The address of the constant product AMM.
 * @param {Address} mint - The token mint address.
 * @returns {Promise<ProgramDerivedAddress>} - The derived address for the AMM vault.
 */
export const getCpAmmVaultPDA = async (cpAmm: Address, mint: Address): Promise<ProgramDerivedAddress> => {
    return await getProgramDerivedAddress({
        programAddress: program.CPMM_PROGRAM_ADDRESS,
        seeds: ["vault", getAddressEncoder().encode(cpAmm), getAddressEncoder().encode(mint)]
    });
};